# Changelog

## v1.0.0 - 2025-07-15

### Added
- Core RFC 2544 test engine (throughput, latency, frameloss, back-to-back)
- AF_XDP zero-copy packet processing
- Multi-queue parallel transmission
- CLI with comprehensive argument support
- JSON, CSV, and HTML output formats
- Y.1564, RFC 2889, RFC 6349, Y.1731, MEF, and TSN test support
- IMIX frame set generation
- Rate limiting (CBR, poison, hardware)
- End-to-end container test suite
- Comprehensive unit test coverage

### Changed
- RFC 2544 binary search algorithm aligned with ByteBlower implementation
- 400G+ optimized packet generation with SIMD-friendly layouts

## v1.1.0 - 2025-10-03

### Added
- TSN gate timing accuracy test
- IMIX payload distribution improvements
- Queue affinity mapping for NUMA-aware scheduling
- Improved JSON report generation with HTML template support

### Fixed
- Frame loss calculation edge case at line rate
- Multi-queue rate scaling for uneven queue counts
- Docker container networking for dual-NIC setups

## v1.2.0 - 2025-11-12

### Added
- YAML configuration file support
- System recovery and reset tests
- Extended RFC 6349 path analysis mode
- Performance benchmark data in docs
- CI/CD pipeline configuration

### Changed
- Optimized AF_XDP ring buffer sizing for 400G
- Improved back-to-back burst algorithm
- Enhanced log output with timing markers

## v1.3.0 - 2025-12-01

### Added
- MEF service activation test
- Y.1731 synthetic loss measurement (SLM)
- Container-based load testing scripts
- README with complete argument reference
- Contributing guidelines

### Fixed
- Frame size boundary cases (64-byte minimum, 9000 jumbo)
- Rate calculation for non-standard line rates
- Docker network configuration for tester/reflector communication
