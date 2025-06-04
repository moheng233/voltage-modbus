# Changelog

All notable changes to Voltage Modbus library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-06-04

### Added
- **Modbus ASCII Transport Support** - Complete implementation of Modbus ASCII protocol
  - `AsciiTransport` class with full ASCII frame encoding/decoding
  - LRC (Longitudinal Redundancy Check) error detection
  - Human-readable frame format for debugging and legacy system integration
  - Configurable serial parameters (7/8 data bits, parity, stop bits)
  - Inter-character timeout handling for ASCII frame reception
  - Comprehensive ASCII frame validation and error handling

### Features
- **ASCII Protocol Implementation**
  - ASCII hex encoding/decoding utilities
  - LRC checksum calculation and verification
  - CR/LF frame termination handling
  - Support for all standard Modbus functions in ASCII format
  - Exception response handling in ASCII format

- **Development Tools**
  - `ascii_test` binary for testing and demonstration
  - ASCII frame logger for debugging purposes
  - Example ASCII frames for educational use
  - Complete test suite for ASCII functionality

### Use Cases
- **Debugging**: Human-readable format for protocol troubleshooting
- **Legacy Systems**: Integration with older SCADA systems that only support ASCII
- **Educational**: Learning Modbus protocol structure with readable format
- **Manual Testing**: Ability to type commands manually in serial terminals

### Updated
- Library documentation to include ASCII transport
- Export statements to include `AsciiTransport`
- Main library description to mention TCP/RTU/ASCII support
- Comprehensive test coverage for ASCII functionality

### Technical Details
- ASCII frames use ':' start character and CR/LF termination
- LRC calculated as two's complement of sum of data bytes
- Default configuration: 7 data bits, even parity, 1 stop bit
- Configurable timeouts for overall operation and inter-character delays
- Full compatibility with existing `ModbusTransport` trait

## [0.1.0] - 2024-06-03

### Added
- Initial release of Voltage Modbus library
- **Modbus TCP Transport** - Complete TCP implementation with MBAP header handling
- **Modbus RTU Transport** - Full RTU implementation with CRC-16 validation
- **Protocol Layer** - Support for all standard Modbus function codes (0x01-0x10)
- **Client/Server Architecture** - Async client and server implementations
- **Register Bank** - Thread-safe register storage for server applications
- **Error Handling** - Comprehensive error types and recovery mechanisms
- **Performance Monitoring** - Built-in statistics and metrics
- **Testing Framework** - Complete test suite and example applications

### Features
- Async/await support with Tokio
- Zero-copy operations where possible
- Thread-safe design for concurrent usage
- Configurable timeouts and retry mechanisms
- Comprehensive logging and debugging support
- Production-ready reliability and performance

### Function Codes Supported

- 0x01: Read Coils
- 0x02: Read Discrete Inputs
- 0x03: Read Holding Registers
- 0x04: Read Input Registers
- 0x05: Write Single Coil
- 0x06: Write Single Register
- 0x0F: Write Multiple Coils
- 0x10: Write Multiple Registers

### Documentation

- Complete API reference with examples
- Architecture documentation and diagrams
- Performance benchmarks and optimization guide
- GitHub Pages deployment for live documentation

### Author

- Evan Liu <evan.liu@voltageenergy.com>
