/// Modbus RTU Test Program
/// 
/// This program demonstrates the clean separation between application layer
/// and network layer in the Modbus RTU implementation.
/// 
/// Architecture:
/// - Application Layer: High-level business logic and data handling
/// - Protocol Layer: Modbus protocol implementation (function codes, etc.)
/// - Transport Layer: RTU-specific network communication (serial, framing, CRC)

use std::time::Duration;
use tokio::time::sleep;
use voltage_modbus::{
    transport::{RtuTransport, ModbusTransport, TransportStats},
    protocol::{ModbusRequest, ModbusResponse, ModbusFunction, ModbusAddress, SlaveId},
    error::{ModbusError, ModbusResult},
    client::ModbusClient,
};

/// Application layer - High-level business logic
pub struct EmsApplication {
    /// Device configurations
    devices: Vec<DeviceConfig>,
    /// Data collection results
    collected_data: std::collections::HashMap<SlaveId, DeviceData>,
}

/// Device configuration in the application layer
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub slave_id: SlaveId,
    pub name: String,
    pub device_type: String,
    pub register_map: RegisterMap,
}

/// Register map defines what data to collect from each device
#[derive(Debug, Clone)]
pub struct RegisterMap {
    pub voltage_registers: Vec<ModbusAddress>,
    pub current_registers: Vec<ModbusAddress>,
    pub power_registers: Vec<ModbusAddress>,
    pub status_coils: Vec<ModbusAddress>,
}

/// Collected device data
#[derive(Debug, Clone, Default)]
pub struct DeviceData {
    pub voltages: Vec<f32>,
    pub currents: Vec<f32>,
    pub powers: Vec<f32>,
    pub status: Vec<bool>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EmsApplication {
    /// Create new EMS application
    pub fn new() -> Self {
        let devices = vec![
            DeviceConfig {
                slave_id: 1,
                name: "Inverter-1".to_string(),
                device_type: "Solar Inverter".to_string(),
                register_map: RegisterMap {
                    voltage_registers: vec![0x0000, 0x0001, 0x0002], // L1, L2, L3 voltage
                    current_registers: vec![0x0010, 0x0011, 0x0012], // L1, L2, L3 current
                    power_registers: vec![0x0020, 0x0021], // Active power, Reactive power
                    status_coils: vec![0x0000, 0x0001, 0x0002], // Running, Fault, Warning
                },
            },
            DeviceConfig {
                slave_id: 2,
                name: "Meter-1".to_string(),
                device_type: "Energy Meter".to_string(),
                register_map: RegisterMap {
                    voltage_registers: vec![0x0100, 0x0101, 0x0102],
                    current_registers: vec![0x0110, 0x0111, 0x0112],
                    power_registers: vec![0x0120, 0x0121, 0x0122],
                    status_coils: vec![0x0100, 0x0101],
                },
            },
        ];

        Self {
            devices,
            collected_data: std::collections::HashMap::new(),
        }
    }

    /// Collect data from all devices using the transport layer
    pub async fn collect_data<T: ModbusTransport>(&mut self, transport: &mut T) -> ModbusResult<()> {
        log::info!("Starting data collection from {} devices", self.devices.len());

        for device in &self.devices {
            log::info!("Collecting data from device: {} (ID: {})", device.name, device.slave_id);
            
            let mut device_data = DeviceData::default();
            device_data.timestamp = chrono::Utc::now();

            // Collect voltage data
            if !device.register_map.voltage_registers.is_empty() {
                device_data.voltages = self.read_float_registers(
                    transport,
                    device.slave_id,
                    &device.register_map.voltage_registers,
                ).await?;
                log::debug!("Voltages: {:?}", device_data.voltages);
            }

            // Collect current data
            if !device.register_map.current_registers.is_empty() {
                device_data.currents = self.read_float_registers(
                    transport,
                    device.slave_id,
                    &device.register_map.current_registers,
                ).await?;
                log::debug!("Currents: {:?}", device_data.currents);
            }

            // Collect power data
            if !device.register_map.power_registers.is_empty() {
                device_data.powers = self.read_float_registers(
                    transport,
                    device.slave_id,
                    &device.register_map.power_registers,
                ).await?;
                log::debug!("Powers: {:?}", device_data.powers);
            }

            // Collect status data
            if !device.register_map.status_coils.is_empty() {
                device_data.status = self.read_coils(
                    transport,
                    device.slave_id,
                    &device.register_map.status_coils,
                ).await?;
                log::debug!("Status: {:?}", device_data.status);
            }

            self.collected_data.insert(device.slave_id, device_data);
            
            // Small delay between devices to avoid overwhelming the network
            sleep(Duration::from_millis(100)).await;
        }

        log::info!("Data collection completed successfully");
        Ok(())
    }

    /// Read floating point values from registers (application layer logic)
    async fn read_float_registers<T: ModbusTransport>(
        &self,
        transport: &mut T,
        slave_id: SlaveId,
        addresses: &[ModbusAddress],
    ) -> ModbusResult<Vec<f32>> {
        let mut values = Vec::new();

        for &address in addresses {
            // Read 2 registers for each float (32-bit)
            let request = ModbusRequest::new_read(
                slave_id,
                ModbusFunction::ReadHoldingRegisters,
                address,
                2, // 2 registers for 32-bit float
            );

            let response = transport.request(&request).await?;
            let registers = response.parse_registers()?;

            if registers.len() >= 2 {
                // Convert 2 registers to float (IEEE 754)
                let combined = ((registers[0] as u32) << 16) | (registers[1] as u32);
                let float_value = f32::from_bits(combined);
                values.push(float_value);
                
                log::trace!("Read float from address 0x{:04X}: {}", address, float_value);
            } else {
                return Err(ModbusError::protocol("Insufficient registers for float conversion"));
            }

            // Small delay between register reads
            sleep(Duration::from_millis(10)).await;
        }

        Ok(values)
    }

    /// Read coil status (application layer logic)
    async fn read_coils<T: ModbusTransport>(
        &self,
        transport: &mut T,
        slave_id: SlaveId,
        addresses: &[ModbusAddress],
    ) -> ModbusResult<Vec<bool>> {
        let mut values = Vec::new();

        for &address in addresses {
            let request = ModbusRequest::new_read(
                slave_id,
                ModbusFunction::ReadCoils,
                address,
                1, // 1 coil
            );

            let response = transport.request(&request).await?;
            let bits = response.parse_bits()?;

            if !bits.is_empty() {
                values.push(bits[0]);
                log::trace!("Read coil from address 0x{:04X}: {}", address, bits[0]);
            }

            sleep(Duration::from_millis(10)).await;
        }

        Ok(values)
    }

    /// Display collected data
    pub fn display_results(&self) {
        println!("\n=== EMS Data Collection Results ===");
        
        for device in &self.devices {
            if let Some(data) = self.collected_data.get(&device.slave_id) {
                println!("\nDevice: {} (Type: {}, ID: {})", 
                    device.name, device.device_type, device.slave_id);
                println!("Timestamp: {}", data.timestamp);
                
                if !data.voltages.is_empty() {
                    println!("Voltages: {:?} V", data.voltages);
                }
                if !data.currents.is_empty() {
                    println!("Currents: {:?} A", data.currents);
                }
                if !data.powers.is_empty() {
                    println!("Powers: {:?} W", data.powers);
                }
                if !data.status.is_empty() {
                    println!("Status: {:?}", data.status);
                }
            }
        }
    }
}

/// Network layer testing - Direct transport layer operations
pub struct NetworkLayerTester {
    transport: RtuTransport,
}

impl NetworkLayerTester {
    /// Create new network layer tester
    pub fn new(port: &str, baud_rate: u32) -> ModbusResult<Self> {
        let transport = RtuTransport::new(port, baud_rate)?;
        Ok(Self { transport })
    }

    /// Test basic RTU frame encoding/decoding
    pub async fn test_basic_communication(&mut self, slave_id: SlaveId) -> ModbusResult<()> {
        log::info!("Testing basic RTU communication with slave {}", slave_id);

        // Test 1: Read holding registers
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadHoldingRegisters,
            0x0000,
            1,
        );

        log::debug!("Sending read holding registers request");
        let response = self.transport.request(&request).await?;
        log::info!("Received response: {:?}", response);

        // Test 2: Read coils
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadCoils,
            0x0000,
            1,
        );

        log::debug!("Sending read coils request");
        let response = self.transport.request(&request).await?;
        log::info!("Received response: {:?}", response);

        // Test 3: Write single register
        let request = ModbusRequest::new_write(
            slave_id,
            ModbusFunction::WriteSingleRegister,
            0x0000,
            vec![0x12, 0x34],
        );

        log::debug!("Sending write single register request");
        let response = self.transport.request(&request).await?;
        log::info!("Received response: {:?}", response);

        Ok(())
    }

    /// Test error handling and edge cases
    pub async fn test_error_handling(&mut self) -> ModbusResult<()> {
        log::info!("Testing error handling scenarios");

        // Test 1: Invalid slave ID
        let request = ModbusRequest::new_read(
            255, // Invalid slave ID
            ModbusFunction::ReadHoldingRegisters,
            0x0000,
            1,
        );

        log::debug!("Testing invalid slave ID");
        match self.transport.request(&request).await {
            Ok(_) => log::warn!("Expected error for invalid slave ID, but got success"),
            Err(e) => log::info!("Expected error for invalid slave ID: {}", e),
        }

        // Test 2: Invalid register address
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            0xFFFF, // High address that might not exist
            100,
        );

        log::debug!("Testing invalid register address");
        match self.transport.request(&request).await {
            Ok(_) => log::info!("High address read succeeded"),
            Err(e) => log::info!("High address read failed (expected): {}", e),
        }

        Ok(())
    }

    /// Display transport statistics
    pub fn display_stats(&self) {
        let stats = self.transport.get_stats();
        println!("\n=== Transport Layer Statistics ===");
        println!("Requests sent: {}", stats.requests_sent);
        println!("Responses received: {}", stats.responses_received);
        println!("Errors: {}", stats.errors);
        println!("Timeouts: {}", stats.timeouts);
        println!("Bytes sent: {}", stats.bytes_sent);
        println!("Bytes received: {}", stats.bytes_received);
        
        if stats.requests_sent > 0 {
            let success_rate = (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0;
            println!("Success rate: {:.2}%", success_rate);
        }
    }
}

/// Test configuration
#[derive(Debug)]
pub struct TestConfig {
    pub port: String,
    pub baud_rate: u32,
    pub test_slave_id: SlaveId,
    pub enable_application_test: bool,
    pub enable_network_test: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            port: "/dev/ttyUSB0".to_string(), // Default Linux serial port
            baud_rate: 9600,
            test_slave_id: 1,
            enable_application_test: true,
            enable_network_test: true,
        }
    }
}

#[tokio::main]
async fn main() -> ModbusResult<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = TestConfig::default();
    
    log::info!("Starting Modbus RTU Test Program");
    log::info!("Configuration: {:?}", config);
    
    println!("=== Modbus RTU Test Program ===");
    println!("This program demonstrates the separation between application and network layers");
    println!("Port: {}, Baud Rate: {}, Slave ID: {}", 
        config.port, config.baud_rate, config.test_slave_id);

    // Network Layer Testing
    if config.enable_network_test {
        println!("\n--- Network Layer Testing ---");
        println!("Testing low-level RTU transport functionality");
        
        match NetworkLayerTester::new(&config.port, config.baud_rate) {
            Ok(mut tester) => {
                // Test basic communication
                if let Err(e) = tester.test_basic_communication(config.test_slave_id).await {
                    log::error!("Basic communication test failed: {}", e);
                    println!("Basic communication test failed: {}", e);
                } else {
                    println!("Basic communication test passed!");
                }

                // Test error handling
                if let Err(e) = tester.test_error_handling().await {
                    log::error!("Error handling test failed: {}", e);
                } else {
                    println!("Error handling test completed!");
                }

                tester.display_stats();
            },
            Err(e) => {
                log::error!("Failed to create network tester: {}", e);
                println!("Failed to create network tester: {}", e);
                println!("Note: Make sure the serial port is available and you have permission to access it");
            }
        }
    }

    // Application Layer Testing  
    if config.enable_application_test {
        println!("\n--- Application Layer Testing ---");
        println!("Testing high-level EMS data collection functionality");
        
        match RtuTransport::new(&config.port, config.baud_rate) {
            Ok(mut transport) => {
                let mut ems_app = EmsApplication::new();
                
                match ems_app.collect_data(&mut transport).await {
                    Ok(_) => {
                        println!("Application layer test passed!");
                        ems_app.display_results();
                    },
                    Err(e) => {
                        log::error!("Application layer test failed: {}", e);
                        println!("Application layer test failed: {}", e);
                    }
                }

                // Display final transport statistics
                let stats = transport.get_stats();
                println!("\n=== Final Transport Statistics ===");
                println!("Total requests: {}", stats.requests_sent);
                println!("Total responses: {}", stats.responses_received);
                println!("Total errors: {}", stats.errors);
            },
            Err(e) => {
                log::error!("Failed to create RTU transport: {}", e);
                println!("Failed to create RTU transport: {}", e);
            }
        }
    }

    println!("\n=== Test Program Completed ===");
    println!("This test demonstrates how the modular architecture separates concerns:");
    println!("- Application Layer: Business logic, data interpretation, device management");
    println!("- Protocol Layer: Modbus protocol implementation, function codes");
    println!("- Transport Layer: RTU-specific communication, serial handling, CRC validation");
    
    Ok(())
} 