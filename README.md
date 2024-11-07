# RISC-V Embassy Project

Welcome to the RISC-V Embassy Project repository! This project aims to provide a comprehensive suite of tools, libraries, and resources for developing and deploying applications on RISC-V architecture. Our goal is to foster a collaborative environment where developers can contribute to and benefit from the growing RISC-V ecosystem.

## Table of Contents
- [Features](#features)
- [Getting Started](#getting-started)
- [Dependencies](#dependencies)
- [Building and Running](#building-and-running)
- [Contributing](#contributing)
- [License](#license)

## Features
* Toolchain Integration: Seamless integration with popular RISC-V toolchains.
* Libraries: A collection of optimized libraries for various applications.
* Documentation: Detailed guides and tutorials to help you get started.
* Community Support: Join our community to share knowledge and get help from other developers.

## Getting Started
1. Clone the Repository: `git clone https://github.com/yourusername/riscv-embassy.git`
2. Install Dependencies: Follow the instructions in the README.md file.
3. Build and Run: Use the provided scripts to build and run your applications.

## Dependencies
1. Install RUST dependencies:
    ```sh
    rustup target add riscv32imac-unknown-none-elf
    rustup target add riscv32imc-unknown-none-elf
    ```
2. For Flashing install Jlink dependencies:
    * For Jlink website download the WIN version:
      https://www.segger.com/downloads/jlink/
    * While using WSL make sure your config file (`C:\Users\<UserName>\.wslconfig`) includes the following configuration:
      ```ini
      # Turn on default connection to bind WSL 2 localhost to Windows localhost
      networkingMode=mirrored
      ```
      !!! * In case `.wslconfig` file does not exist, please create it as follows:
      ```ini
      # Settings apply across all Linux distros running on WSL 2
      [wsl2]

      # Turn on default connection to bind WSL 2 localhost to Windows localhost
      networkingMode=mirrored
      ```

## Building and Running
### Building
1. Use the build task for building the project.

### Flashing
1. Make sure to install Jlink dependencies from the previous section.
2. For running Jlink server, run the following command in Windows CMD/Powershell:
    ```sh
    JLinkGDBServer -device FE310 -if JTAG -speed 4000 -port 3333
    ```
3. Run the Flash task.

### Debug
1. Use QEMU for debugging purposes. Just run the `QEMU-RISCV launch` from the debug label.
2. Use Red-V HW for debugging purposes:
    * Make sure Jlink server is running (command in the previous section).
    * Run the `Red-V launch` from the debug label.

## Contributing
We welcome contributions from the community! Please read our contributing guidelines to get started.

## License
This project is licensed under the MIT License. See the LICENSE file for more details.
