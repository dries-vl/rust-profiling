

	@WSL

	1: wsl -l -v  													// to see all your currently installed distros and which version of WSL
	2: wsl --list --online  										// to see all available distros
	3: wsl --install -d Ubuntu-22.04  								// install
	
	4: wsl.exe -d Ubuntu-22.04  									// switch to a distro
	5: C:\Windows\System32\wslconfig /setdefault Ubuntu-22.04		// set default distro
	6: wsl --set-version Ubuntu-22.04 2								// set distro wsl version
	7: wsl --export <Distribution Name> <FileName>
	8: wsl --import <Distribution Name> <InstallLocation> <FileName>	// import and export (.tar)
	
	
	@C AND C++
	
	sudo apt update 												// update the list of available packages
	sudo apt-get install build-essential							// install gnu compilers for c, c++, go, fortran, obj-c
	sudo apt-get install gdb										// install gnu debugger for c, c++, go, rust
	sudo apt install mingw-w64										// install mingw to compile c, c++ to windows
	
	@RUST
	
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh	// install rustup -> rust and cargo
	