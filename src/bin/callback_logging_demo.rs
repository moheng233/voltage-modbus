use std::time::Duration;
use voltage_modbus::{
    client::{ModbusTcpClient, ModbusClient},
    logging::{CallbackLogger, LogLevel, LogCallback, LoggingMode},
    server::{ModbusTcpServer, ModbusServer},
    register_bank::ModbusRegisterBank,
    ModbusResult,
};
use std::sync::Arc;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> ModbusResult<()> {
    println!("🚀 Voltage Modbus - Callback Logging Demo");
    println!("==========================================\n");

    // Start a test server
    println!("🔧 Starting test server...");
    let register_bank = Arc::new(ModbusRegisterBank::new());
    
    // Initialize some test data
    for i in 0..10 {
        register_bank.write_06(i, 1000 + i).unwrap();
        register_bank.write_05(i, i % 2 == 0).unwrap();
    }
    
    let mut server = ModbusTcpServer::new("127.0.0.1:5502")?;
    server.set_register_bank(register_bank);
    
    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });
    
    // Wait for server to start
    sleep(Duration::from_millis(500)).await;
    
    println!("📋 Demo 1: Interpreted Mode (Default)");
    println!("=====================================");
    
    // Create custom log callback with different formatting
    let interpreted_callback: LogCallback = Box::new(|level, message| {
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
        let level_emoji = match level {
            LogLevel::Error => "❌",
            LogLevel::Warn => "⚠️",
            LogLevel::Info => "📋",
            LogLevel::Debug => "🔍",
        };
        println!("[{}] {} [{}] {}", timestamp, level_emoji, level.as_str(), message);
    });

    // Create a logger with interpreted mode
    let logger = CallbackLogger::with_mode(Some(interpreted_callback), LogLevel::Info, LoggingMode::Interpreted);

    // Create client with interpreted logging
    let mut client = ModbusTcpClient::with_logging(
        "127.0.0.1:5502", 
        Duration::from_secs(1), 
        Some(logger)
    ).await?;

    // Test read operations
    match client.read_03(1, 0, 5).await {
        Ok(values) => {
            println!("✅ Read holding registers: {:?}\n", values);
        }
        Err(e) => {
            println!("❌ Failed to read holding registers: {}\n", e);
        }
    }

    match client.write_06(1, 10, 0xABCD).await {
        Ok(_) => {
            println!("✅ Wrote single register\n");
        }
        Err(e) => {
            println!("❌ Failed to write register: {}\n", e);
        }
    }

    sleep(Duration::from_millis(500)).await;

    println!("📤 Demo 2: Raw Mode");
    println!("===================");

    // Create logger with raw mode
    let raw_callback: LogCallback = Box::new(|level, message| {
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
        match level {
            LogLevel::Error => eprintln!("[{}] 🔴 ERROR: {}", timestamp, message),
            LogLevel::Warn => eprintln!("[{}] 🟡 WARN: {}", timestamp, message),
            LogLevel::Info => println!("[{}] 📤 {}", timestamp, message),
            LogLevel::Debug => println!("[{}] 🔍 DEBUG: {}", timestamp, message),
        }
    });

    let raw_logger = CallbackLogger::with_mode(Some(raw_callback), LogLevel::Info, LoggingMode::Raw);
    let mut client2 = ModbusTcpClient::with_logging(
        "127.0.0.1:5502", 
        Duration::from_secs(1), 
        Some(raw_logger)
    ).await?;

    // Test with raw logging
    match client2.read_01(1, 0, 8).await {
        Ok(coils) => {
            println!("✅ Read coils: {:?}\n", coils);
        }
        Err(e) => {
            println!("❌ Failed to read coils: {}\n", e);
        }
    }

    match client2.write_05(1, 15, true).await {
        Ok(_) => {
            println!("✅ Wrote single coil\n");
        }
        Err(e) => {
            println!("❌ Failed to write coil: {}\n", e);
        }
    }

    sleep(Duration::from_millis(500)).await;

    println!("📊 Demo 3: Both Mode (Interpreted + Raw)");
    println!("=========================================");

    // Create logger with both mode
    let both_callback: LogCallback = Box::new(|level, message| {
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
        match level {
            LogLevel::Error => eprintln!("[{}] 🔴 ERROR: {}", timestamp, message),
            LogLevel::Warn => eprintln!("[{}] 🟡 WARN: {}", timestamp, message),
            LogLevel::Info => println!("[{}] 📊 {}", timestamp, message),
            LogLevel::Debug => println!("[{}] 🔍 {}", timestamp, message),
        }
    });

    let both_logger = CallbackLogger::with_mode(Some(both_callback), LogLevel::Debug, LoggingMode::Both);
    let mut client3 = ModbusTcpClient::with_logging(
        "127.0.0.1:5502", 
        Duration::from_secs(1), 
        Some(both_logger)
    ).await?;

    // Test with both modes
    match client3.write_10(1, 20, &[0x1111, 0x2222, 0x3333]).await {
        Ok(_) => {
            println!("✅ Wrote multiple registers\n");
        }
        Err(e) => {
            println!("❌ Failed to write multiple registers: {}\n", e);
        }
    }

    match client3.read_03(1, 20, 3).await {
        Ok(values) => {
            println!("✅ Read back values: {:?}\n", values);
        }
        Err(e) => {
            println!("❌ Failed to read back values: {}\n", e);
        }
    }

    println!("🔇 Demo 4: Disabled Logging");
    println!("===========================");
    
    // Create client with disabled logging
    let mut client4 = ModbusTcpClient::with_logging(
        "127.0.0.1:5502", 
        Duration::from_secs(1), 
        None // No logging
    ).await?;

    // Test with no logging
    match client4.read_03(1, 0, 2).await {
        Ok(values) => {
            println!("No logging test - Read values: {:?}\n", values);
        }
        Err(e) => {
            println!("No logging test failed: {}\n", e);
        }
    }

    println!("✅ Callback logging demo completed!");
    println!("\n📝 Summary:");
    println!("   - Interpreted Mode: Human-readable function names and data interpretation");
    println!("   - Raw Mode: Complete hex packet data for protocol analysis");
    println!("   - Both Mode: Interpreted at INFO level, raw at DEBUG level");
    println!("   - Disabled Mode: No packet logging");
    
    Ok(())
} 