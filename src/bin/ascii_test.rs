/// Modbus ASCII Transport Test Program
/// 
/// This program demonstrates the usage of Modbus ASCII transport for
/// debugging and integration with legacy systems.
/// 
/// ASCII format is human-readable and useful for:
/// - Protocol debugging and troubleshooting
/// - Integration with legacy SCADA systems
/// - Educational purposes and learning
/// - Manual testing with serial terminals

use voltage_modbus::transport::{AsciiTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::{ModbusError, ModbusResult};
use std::env;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use log::{info, warn, error, debug};

/// ASCII test configuration
struct AsciiTestConfig {
    port: String,
    baud_rate: u32,
    slave_id: u8,
    test_address: u16,
    test_quantity: u16,
}

impl AsciiTestConfig {
    /// Load configuration from environment variables or use defaults
    fn from_env() -> Self {
        Self {
            port: env::var("MODBUS_ASCII_PORT").unwrap_or_else(|_| "/dev/ttyUSB0".to_string()),
            baud_rate: env::var("MODBUS_ASCII_BAUD")
                .unwrap_or_else(|_| "9600".to_string())
                .parse()
                .unwrap_or(9600),
            slave_id: env::var("MODBUS_SLAVE_ID")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            test_address: env::var("MODBUS_TEST_ADDRESS")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
            test_quantity: env::var("MODBUS_TEST_QUANTITY")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// ASCII Frame Logger - demonstrates the human-readable format
struct AsciiFrameLogger;

impl AsciiFrameLogger {
    /// Log ASCII frame in human-readable format
    fn log_frame(direction: &str, frame: &[u8]) {
        let frame_str = String::from_utf8_lossy(frame);
        info!("{}: {}", direction, frame_str);
        
        // Parse frame if it looks like ASCII
        if frame.len() > 1 && frame[0] == b':' {
            Self::parse_and_log_frame(&frame_str);
        }
    }
    
    /// Parse and explain ASCII frame structure
    fn parse_and_log_frame(frame: &str) {
        if frame.len() < 11 {
            warn!("ASCII frame too short: {}", frame);
            return;
        }
        
        info!("ASCII Frame Structure:");
        info!("  Start: '{}' (0x{:02X})", frame.chars().nth(0).unwrap(), frame.as_bytes()[0]);
        
        if let (Some(addr_h), Some(addr_l)) = (frame.chars().nth(1), frame.chars().nth(2)) {
            info!("  Address: {}{} (0x{}{})", addr_h, addr_l, addr_h, addr_l);
        }
        
        if let (Some(func_h), Some(func_l)) = (frame.chars().nth(3), frame.chars().nth(4)) {
            info!("  Function: {}{} (0x{}{})", func_h, func_l, func_h, func_l);
        }
        
        let data_len = frame.len() - 9; // Start(1) + Addr(2) + Func(2) + LRC(2) + CRLF(2)
        if data_len > 0 {
            let data_part = &frame[5..5 + data_len];
            info!("  Data: {} ({} chars)", data_part, data_len);
        }
        
        if frame.len() >= 7 {
            let lrc_part = &frame[frame.len() - 4..frame.len() - 2];
            info!("  LRC: {} (0x{})", lrc_part, lrc_part);
        }
        
        info!("  End: CR LF (0x0D 0x0A)");
    }
}

/// ASCII Transport Test Suite
struct AsciiTestSuite {
    transport: AsciiTransport,
    config: AsciiTestConfig,
}

impl AsciiTestSuite {
    /// Create new test suite
    async fn new(config: AsciiTestConfig) -> ModbusResult<Self> {
        info!("Creating ASCII transport: {} @ {} baud", config.port, config.baud_rate);
        
        let transport = AsciiTransport::new_with_config(
            &config.port,
            config.baud_rate,
            tokio_serial::DataBits::Seven,  // ASCII standard
            tokio_serial::StopBits::One,
            tokio_serial::Parity::Even,     // Recommended for ASCII
            Duration::from_secs(5),         // Operation timeout
            Duration::from_millis(100),     // Inter-character timeout
        )?;
        
        Ok(Self { transport, config })
    }
    
    /// Test ASCII frame encoding and communication
    async fn test_ascii_communication(&mut self) -> ModbusResult<()> {
        info!("=== ASCII Communication Test ===");
        
        // Test 1: Read Holding Registers
        info!("Test 1: Reading {} holding registers from address {}", 
              self.config.test_quantity, self.config.test_address);
        
        let read_request = ModbusRequest::new_read(
            self.config.slave_id,
            ModbusFunction::ReadHoldingRegisters,
            self.config.test_address,
            self.config.test_quantity,
        );
        
        match self.transport.request(&read_request).await {
            Ok(response) => {
                info!("✅ Read request successful");
                if !response.is_exception() {
                    let registers = response.parse_registers()?;
                    info!("Read {} registers: {:?}", registers.len(), registers);
                    
                    // Display in hex format for clarity
                    let hex_values: Vec<String> = registers.iter()
                        .map(|&v| format!("0x{:04X}", v))
                        .collect();
                    info!("Hex format: [{}]", hex_values.join(", "));
                } else {
                    warn!("Received exception response");
                }
            },
            Err(e) => {
                error!("❌ Read request failed: {}", e);
            }
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Test 2: Read Coils
        info!("Test 2: Reading 8 coils from address 0");
        
        let coil_request = ModbusRequest::new_read(
            self.config.slave_id,
            ModbusFunction::ReadCoils,
            0,
            8,
        );
        
        match self.transport.request(&coil_request).await {
            Ok(response) => {
                info!("✅ Coil read successful");
                if !response.is_exception() {
                    let bits = response.parse_bits()?;
                    info!("Read {} coils: {:?}", bits.len().min(8), &bits[..bits.len().min(8)]);
                }
            },
            Err(e) => {
                warn!("Coil read failed (may be expected): {}", e);
            }
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Test 3: Write Single Register (if supported)
        info!("Test 3: Writing single register");
        
        let write_request = ModbusRequest::new_write(
            self.config.slave_id,
            ModbusFunction::WriteSingleRegister,
            100,
            vec![0x12, 0x34],
        );
        
        match self.transport.request(&write_request).await {
            Ok(_response) => {
                info!("✅ Write request successful");
            },
            Err(e) => {
                warn!("Write request failed (may be expected): {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Demonstrate ASCII format advantages
    async fn demonstrate_ascii_features(&mut self) -> ModbusResult<()> {
        info!("=== ASCII Format Demonstration ===");
        
        info!("ASCII frames are human-readable text format:");
        info!("Example: ':010300000002C5\\r\\n'");
        info!("  Start: ':'");
        info!("  Address: '01' (slave 1)");
        info!("  Function: '03' (read holding registers)");
        info!("  Data: '00000002' (address 0, quantity 2)");
        info!("  LRC: 'C5' (checksum)");
        info!("  End: '\\r\\n' (CR LF)");
        
        info!("Advantages of ASCII format:");
        info!("  🔍 Human-readable for debugging");
        info!("  📝 Can be typed manually in terminal");
        info!("  🔧 Easy integration with legacy systems");
        info!("  📚 Educational value for learning protocol");
        info!("  🛠️ Simple to parse with text tools");
        
        info!("Disadvantages compared to RTU:");
        info!("  📦 ~2x larger frame size");
        info!("  ⏱️ Slower transmission");
        info!("  🔋 Higher power consumption");
        info!("  ⚡ LRC is weaker than CRC-16");
        
        Ok(())
    }
    
    /// Display transport statistics
    fn display_statistics(&self) {
        let stats = self.transport.get_stats();
        
        info!("=== ASCII Transport Statistics ===");
        info!("Requests sent: {}", stats.requests_sent);
        info!("Responses received: {}", stats.responses_received);
        info!("Errors: {}", stats.errors);
        info!("Timeouts: {}", stats.timeouts);
        info!("Bytes sent: {}", stats.bytes_sent);
        info!("Bytes received: {}", stats.bytes_received);
        
        if stats.requests_sent > 0 {
            let success_rate = (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0;
            info!("Success rate: {:.2}%", success_rate);
            
            let avg_frame_size = stats.bytes_sent as f64 / stats.requests_sent as f64;
            info!("Average frame size: {:.1} bytes", avg_frame_size);
        }
    }
}

/// Manual ASCII frame testing
async fn manual_ascii_test() -> ModbusResult<()> {
    info!("=== Manual ASCII Frame Test ===");
    info!("This test demonstrates ASCII frame format without actual hardware");
    
    // Example ASCII frames
    let examples = [
        (":010300000002C5\r\n", "Read 2 holding registers from address 0"),
        (":010100000008F7\r\n", "Read 8 coils from address 0"),
        (":01060064003CE6\r\n", "Write 0x003C to register 100"),
        (":010F000000083BC7\r\n", "Write coils starting at address 0"),
    ];
    
    for (frame, description) in &examples {
        info!("Frame: {} - {}", frame.trim(), description);
        AsciiFrameLogger::log_frame("Example", frame.as_bytes());
        info!(""); // Empty line for readability
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("=== Modbus ASCII Transport Test ===");
    info!("Testing ASCII format for debugging and legacy system integration");
    
    let config = AsciiTestConfig::from_env();
    
    info!("Configuration:");
    info!("  Port: {}", config.port);
    info!("  Baud Rate: {}", config.baud_rate);
    info!("  Slave ID: {}", config.slave_id);
    info!("  Test Address: {}", config.test_address);
    info!("  Test Quantity: {}", config.test_quantity);
    info!("");
    
    // Always run manual test first
    manual_ascii_test().await?;
    
    // Try hardware test if possible
    match AsciiTestSuite::new(config).await {
        Ok(mut test_suite) => {
            info!("ASCII transport created successfully");
            
            // Run communication tests
            test_suite.demonstrate_ascii_features().await?;
            
            if let Err(e) = test_suite.test_ascii_communication().await {
                warn!("Hardware communication test failed: {}", e);
                info!("This is expected if no ASCII device is connected");
            }
            
            test_suite.display_statistics();
            
            // Close transport
            test_suite.transport.close().await?;
            info!("Transport closed");
        },
        Err(e) => {
            warn!("Failed to create ASCII transport: {}", e);
            info!("This is expected if serial port is not available");
            info!("Manual frame examples were still demonstrated above");
        }
    }
    
    info!("=== ASCII Test Complete ===");
    info!("Key ASCII use cases:");
    info!("  • Debugging protocol issues");
    info!("  • Legacy system integration");
    info!("  • Educational and training");
    info!("  • Manual testing with terminals");
    
    Ok(())
} 