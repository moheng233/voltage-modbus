/// Voltage Modbus Demo
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// This program demonstrates basic usage of the voltage_modbus library.

use std::time::Duration;
use tokio::time::sleep;
use voltage_modbus::{ModbusClient, ModbusTcpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Voltage Modbus Demo");
    println!("=====================");
    
    let server_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:502".to_string());
    
    println!("Connecting to Modbus server at {}...", server_address);
    
    let timeout = Duration::from_secs(5);
    
    // Parse address
    let address: std::net::SocketAddr = server_address.parse()
        .map_err(|e| format!("Invalid server address: {}", e))?;
    
    let mut client = match ModbusTcpClient::from_address(&server_address, timeout).await {
        Ok(client) => {
            println!("✅ Connected successfully!");
            client
        },
        Err(e) => {
            eprintln!("❌ Failed to connect: {}", e);
            eprintln!("Make sure a Modbus server is running on {}", server_address);
            return Ok(());
        }
    };
    
    let slave_id = 1;
    
    println!("\n📖 Testing read operations...");
    
    // Test reading holding registers
    match client.read_03(slave_id, 100, 5).await {
        Ok(values) => {
            println!("📈 Read holding registers 100-104: {:?}", values);
            for (i, value) in values.iter().enumerate() {
                println!("  Register {}: {} (0x{:04X})", 100 + i, value, value);
            }
        },
        Err(e) => println!("❌ Failed to read holding registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading input registers
    match client.read_04(slave_id, 200, 3).await {
        Ok(values) => {
            println!("📊 Read input registers 200-202: {:?}", values);
        },
        Err(e) => println!("❌ Failed to read input registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading coils
    match client.read_01(slave_id, 0, 8).await {
        Ok(coils) => {
            println!("💡 Read coils 0-7: {:?}", coils);
            for (i, &coil) in coils.iter().enumerate() {
                println!("  Coil {}: {}", i, if coil { "ON" } else { "OFF" });
            }
        },
        Err(e) => println!("❌ Failed to read coils: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading discrete inputs
    match client.read_02(slave_id, 100, 4).await {
        Ok(inputs) => {
            println!("🔌 Read discrete inputs 100-103: {:?}", inputs);
        },
        Err(e) => println!("❌ Failed to read discrete inputs: {}", e),
    }
    
    println!("\n✏️  Testing write operations...");
    
    // Write single register using function code 0x06
    match client.write_06(slave_id, 300, 0xABCD).await {
        Ok(_) => println!("✅ Wrote single register 300 = 0xABCD"),
        Err(e) => println!("❌ Failed to write single register: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Write single coil using function code 0x05
    match client.write_05(slave_id, 100, true).await {
        Ok(_) => println!("✅ Wrote single coil 100 = true"),
        Err(e) => println!("❌ Failed to write single coil: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Write multiple registers using function code 0x10
    match client.write_10(slave_id, 400, &[0x1111, 0x2222, 0x3333]).await {
        Ok(_) => println!("✅ Wrote multiple registers 400-402"),
        Err(e) => println!("❌ Failed to write multiple registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Write multiple coils using function code 0x0F
    match client.write_0f(slave_id, 200, &[true, false, true, false]).await {
        Ok(_) => println!("✅ Wrote multiple coils 200-203"),
        Err(e) => println!("❌ Failed to write multiple coils: {}", e),
    }
    
    // Show connection statistics
    let stats = client.get_stats();
    println!("\n📊 Connection Statistics:");
    println!("   Requests sent: {}", stats.requests_sent);
    println!("   Responses received: {}", stats.responses_received);
    println!("   Errors: {}", stats.errors);
    println!("   Timeouts: {}", stats.timeouts);
    println!("   Bytes sent: {}", stats.bytes_sent);
    println!("   Bytes received: {}", stats.bytes_received);
    
    if stats.requests_sent > 0 {
        let success_rate = (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0;
        println!("   Success rate: {:.1}%", success_rate);
    }
    
    // Close connection
    if let Err(e) = client.close().await {
        eprintln!("⚠️  Error closing connection: {}", e);
    } else {
        println!("\n✅ Connection closed successfully");
    }
    
    println!("\n🎉 Demo completed!");
    println!("👋 Thank you for using Voltage Modbus by Evan Liu!");
    
    Ok(())
} 