/// Modbus RTU Simulator
/// 
/// This simulator creates a virtual serial port pair and acts as a Modbus RTU slave device.
/// It can be used to test the RTU implementation without requiring physical hardware.
/// 
/// The simulator supports:
/// - Basic read/write operations
/// - Multiple register banks
/// - Exception responses
/// - Configurable device behavior

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use rand;
use voltage_modbus::{
    transport::RtuTransport,
    protocol::{ModbusRequest, ModbusResponse, ModbusFunction, ModbusException, SlaveId},
    error::{ModbusError, ModbusResult},
};

/// Virtual Modbus RTU device simulator
pub struct RtuSimulator {
    /// Device slave ID
    slave_id: SlaveId,
    /// Device name
    device_name: String,
    /// Register banks for different data types
    holding_registers: HashMap<u16, u16>,
    input_registers: HashMap<u16, u16>,
    coils: HashMap<u16, bool>,
    discrete_inputs: HashMap<u16, bool>,
    /// Device status
    is_running: bool,
    /// Simulation parameters
    response_delay: Duration,
    error_rate: f32, // 0.0 to 1.0
}

impl RtuSimulator {
    /// Create a new RTU simulator
    pub fn new(slave_id: SlaveId, device_name: String) -> Self {
        let mut simulator = Self {
            slave_id,
            device_name,
            holding_registers: HashMap::new(),
            input_registers: HashMap::new(),
            coils: HashMap::new(),
            discrete_inputs: HashMap::new(),
            is_running: false,
            response_delay: Duration::from_millis(50),
            error_rate: 0.0,
        };
        
        // Initialize with some default values
        simulator.initialize_default_data();
        simulator
    }
    
    /// Initialize device with default test data
    fn initialize_default_data(&mut self) {
        // Holding registers - voltage readings (simulated)
        for i in 0..10 {
            let voltage = 220.0 + (i as f32 * 0.5); // 220V to 224.5V
            let voltage_bits = voltage.to_bits();
            self.holding_registers.insert(i * 2, (voltage_bits >> 16) as u16);
            self.holding_registers.insert(i * 2 + 1, voltage_bits as u16);
        }
        
        // Current readings (simulated)
        for i in 10..20 {
            let current = 10.0 + (i as f32 * 0.1); // 11A to 11.9A
            let current_bits = current.to_bits();
            self.holding_registers.insert(i * 2, (current_bits >> 16) as u16);
            self.holding_registers.insert(i * 2 + 1, current_bits as u16);
        }
        
        // Power readings
        for i in 20..30 {
            let power = 2200.0 + (i as f32 * 10.0); // 2200W to 2290W
            let power_bits = power.to_bits();
            self.holding_registers.insert(i * 2, (power_bits >> 16) as u16);
            self.holding_registers.insert(i * 2 + 1, power_bits as u16);
        }
        
        // Input registers (read-only sensor data)
        for i in 0..50 {
            self.input_registers.insert(i, 1000 + i);
        }
        
        // Coils (control outputs)
        for i in 0..100 {
            self.coils.insert(i, i % 3 == 0); // Every 3rd coil is ON
        }
        
        // Discrete inputs (status inputs)
        for i in 0..100 {
            self.discrete_inputs.insert(i, i % 2 == 0); // Every other input is ON
        }
        
        log::info!("Initialized {} with default data", self.device_name);
    }
    
    /// Process a Modbus request and generate response
    pub fn process_request(&mut self, request: &ModbusRequest) -> ModbusResult<ModbusResponse> {
        // Check if request is for this device
        if request.slave_id != self.slave_id {
            return Err(ModbusError::protocol("Request not for this device"));
        }
        
        // Simulate response delay
        std::thread::sleep(self.response_delay);
        
        // Simulate random errors
        if self.error_rate > 0.0 && rand::random::<f32>() < self.error_rate {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                request.function,
                ModbusException::ServerDeviceFailure.to_u8(),
            ));
        }
        
        // Process based on function code
        match request.function {
            ModbusFunction::ReadHoldingRegisters => {
                self.read_holding_registers(request.address, request.quantity)
            },
            ModbusFunction::ReadInputRegisters => {
                self.read_input_registers(request.address, request.quantity)
            },
            ModbusFunction::ReadCoils => {
                self.read_coils(request.address, request.quantity)
            },
            ModbusFunction::ReadDiscreteInputs => {
                self.read_discrete_inputs(request.address, request.quantity)
            },
            ModbusFunction::WriteSingleRegister => {
                self.write_single_register(request.address, &request.data)
            },
            ModbusFunction::WriteSingleCoil => {
                self.write_single_coil(request.address, &request.data)
            },
            ModbusFunction::WriteMultipleRegisters => {
                self.write_multiple_registers(request.address, request.quantity, &request.data)
            },
            ModbusFunction::WriteMultipleCoils => {
                self.write_multiple_coils(request.address, request.quantity, &request.data)
            },
        }
    }
    
    /// Read holding registers
    fn read_holding_registers(&self, address: u16, quantity: u16) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 125 {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::ReadHoldingRegisters,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let mut data = Vec::new();
        for i in 0..quantity {
            let reg_addr = address.wrapping_add(i);
            let value = self.holding_registers.get(&reg_addr).cloned().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        log::debug!("Read {} holding registers from address 0x{:04X}", quantity, address);
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::ReadHoldingRegisters,
            data,
        ))
    }
    
    /// Read input registers
    fn read_input_registers(&self, address: u16, quantity: u16) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 125 {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::ReadInputRegisters,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let mut data = Vec::new();
        for i in 0..quantity {
            let reg_addr = address.wrapping_add(i);
            let value = self.input_registers.get(&reg_addr).cloned().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        log::debug!("Read {} input registers from address 0x{:04X}", quantity, address);
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::ReadInputRegisters,
            data,
        ))
    }
    
    /// Read coils
    fn read_coils(&self, address: u16, quantity: u16) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 2000 {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::ReadCoils,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let mut bits = Vec::new();
        for i in 0..quantity {
            let coil_addr = address.wrapping_add(i);
            let value = self.coils.get(&coil_addr).cloned().unwrap_or(false);
            bits.push(value);
        }
        
        // Pack bits into bytes
        let mut data = Vec::new();
        let byte_count = (quantity + 7) / 8;
        data.push(byte_count as u8);
        
        for byte_idx in 0..byte_count {
            let mut byte_val = 0u8;
            for bit_idx in 0..8 {
                let bit_pos = byte_idx * 8 + bit_idx;
                if bit_pos < quantity && bit_pos < bits.len() as u16 {
                    if bits[bit_pos as usize] {
                        byte_val |= 1 << bit_idx;
                    }
                }
            }
            data.push(byte_val);
        }
        
        log::debug!("Read {} coils from address 0x{:04X}", quantity, address);
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::ReadCoils,
            data,
        ))
    }
    
    /// Read discrete inputs
    fn read_discrete_inputs(&self, address: u16, quantity: u16) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 2000 {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::ReadDiscreteInputs,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let mut bits = Vec::new();
        for i in 0..quantity {
            let input_addr = address.wrapping_add(i);
            let value = self.discrete_inputs.get(&input_addr).cloned().unwrap_or(false);
            bits.push(value);
        }
        
        // Pack bits into bytes
        let mut data = Vec::new();
        let byte_count = (quantity + 7) / 8;
        data.push(byte_count as u8);
        
        for byte_idx in 0..byte_count {
            let mut byte_val = 0u8;
            for bit_idx in 0..8 {
                let bit_pos = byte_idx * 8 + bit_idx;
                if bit_pos < quantity && bit_pos < bits.len() as u16 {
                    if bits[bit_pos as usize] {
                        byte_val |= 1 << bit_idx;
                    }
                }
            }
            data.push(byte_val);
        }
        
        log::debug!("Read {} discrete inputs from address 0x{:04X}", quantity, address);
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::ReadDiscreteInputs,
            data,
        ))
    }
    
    /// Write single register
    fn write_single_register(&mut self, address: u16, data: &[u8]) -> ModbusResult<ModbusResponse> {
        if data.len() < 2 {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::WriteSingleRegister,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let value = u16::from_be_bytes([data[0], data[1]]);
        self.holding_registers.insert(address, value);
        
        log::debug!("Wrote register 0x{:04X} = 0x{:04X}", address, value);
        
        // Echo back the address and value
        let mut response_data = Vec::new();
        response_data.extend_from_slice(&address.to_be_bytes());
        response_data.extend_from_slice(&value.to_be_bytes());
        
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::WriteSingleRegister,
            response_data,
        ))
    }
    
    /// Write single coil
    fn write_single_coil(&mut self, address: u16, data: &[u8]) -> ModbusResult<ModbusResponse> {
        if data.is_empty() {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::WriteSingleCoil,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        let value = data[0] != 0;
        self.coils.insert(address, value);
        
        log::debug!("Wrote coil 0x{:04X} = {}", address, value);
        
        // Echo back the address and value
        let mut response_data = Vec::new();
        response_data.extend_from_slice(&address.to_be_bytes());
        let coil_value: u16 = if value { 0xFF00 } else { 0x0000 };
        response_data.extend_from_slice(&coil_value.to_be_bytes());
        
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::WriteSingleCoil,
            response_data,
        ))
    }
    
    /// Write multiple registers
    fn write_multiple_registers(&mut self, address: u16, quantity: u16, data: &[u8]) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 123 || data.len() < (quantity * 2) as usize {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::WriteMultipleRegisters,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        // Skip byte count (first byte)
        let reg_data = if data.len() > 1 { &data[1..] } else { data };
        
        for i in 0..quantity {
            let reg_addr = address.wrapping_add(i);
            let data_idx = (i * 2) as usize;
            if data_idx + 1 < reg_data.len() {
                let value = u16::from_be_bytes([reg_data[data_idx], reg_data[data_idx + 1]]);
                self.holding_registers.insert(reg_addr, value);
            }
        }
        
        log::debug!("Wrote {} registers starting from address 0x{:04X}", quantity, address);
        
        // Response contains starting address and quantity
        let mut response_data = Vec::new();
        response_data.extend_from_slice(&address.to_be_bytes());
        response_data.extend_from_slice(&quantity.to_be_bytes());
        
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::WriteMultipleRegisters,
            response_data,
        ))
    }
    
    /// Write multiple coils
    fn write_multiple_coils(&mut self, address: u16, quantity: u16, data: &[u8]) -> ModbusResult<ModbusResponse> {
        if quantity == 0 || quantity > 1968 || data.is_empty() {
            return Ok(ModbusResponse::new_exception(
                self.slave_id,
                ModbusFunction::WriteMultipleCoils,
                ModbusException::IllegalDataValue.to_u8(),
            ));
        }
        
        // Skip byte count (first byte)
        let bit_data = if data.len() > 1 { &data[1..] } else { data };
        
        for i in 0..quantity {
            let coil_addr = address.wrapping_add(i);
            let byte_idx = (i / 8) as usize;
            let bit_idx = i % 8;
            
            if byte_idx < bit_data.len() {
                let value = (bit_data[byte_idx] & (1 << bit_idx)) != 0;
                self.coils.insert(coil_addr, value);
            }
        }
        
        log::debug!("Wrote {} coils starting from address 0x{:04X}", quantity, address);
        
        // Response contains starting address and quantity
        let mut response_data = Vec::new();
        response_data.extend_from_slice(&address.to_be_bytes());
        response_data.extend_from_slice(&quantity.to_be_bytes());
        
        Ok(ModbusResponse::new_success(
            self.slave_id,
            ModbusFunction::WriteMultipleCoils,
            response_data,
        ))
    }
    
    /// Set error simulation rate
    pub fn set_error_rate(&mut self, rate: f32) {
        self.error_rate = rate.clamp(0.0, 1.0);
        log::info!("Set error rate to {:.1}%", self.error_rate * 100.0);
    }
    
    /// Set response delay
    pub fn set_response_delay(&mut self, delay: Duration) {
        self.response_delay = delay;
        log::info!("Set response delay to {:?}", self.response_delay);
    }
    
    /// Get device status
    pub fn get_status(&self) -> String {
        format!("Device: {} (ID: {}), Registers: {}, Coils: {}, Running: {}",
            self.device_name,
            self.slave_id,
            self.holding_registers.len(),
            self.coils.len(),
            self.is_running
        )
    }
}

/// Simple test function to demonstrate the simulator
async fn test_simulator() -> ModbusResult<()> {
    println!("=== RTU Simulator Test ===");
    
    // Create a simulator
    let mut simulator = RtuSimulator::new(1, "Test Device".to_string());
    
    // Test read holding registers
    let request = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        0x0000,
        5,
    );
    
    let response = simulator.process_request(&request)?;
    println!("Read response: {:?}", response);
    
    // Test write single register
    let request = ModbusRequest::new_write(
        1,
        ModbusFunction::WriteSingleRegister,
        0x0100,
        vec![0x12, 0x34],
    );
    
    let response = simulator.process_request(&request)?;
    println!("Write response: {:?}", response);
    
    // Test read coils
    let request = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadCoils,
        0x0000,
        10,
    );
    
    let response = simulator.process_request(&request)?;
    println!("Coils response: {:?}", response);
    
    println!("Simulator status: {}", simulator.get_status());
    
    Ok(())
}

#[tokio::main]
async fn main() -> ModbusResult<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::info!("Starting Modbus RTU Simulator");
    
    test_simulator().await?;
    
    println!("\n=== Simulator Completed ===");
    println!("This simulator can be used to test RTU functionality without physical hardware.");
    println!("In a real application, you would:");
    println!("1. Create virtual serial ports (using socat or similar tools)");
    println!("2. Run the simulator on one end");
    println!("3. Connect your RTU client to the other end");
    
    Ok(())
} 