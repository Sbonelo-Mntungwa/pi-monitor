Vagrant.configure("2") do |config|
  config.vm.box = "spox/ubuntu-arm"
  config.vm.define "pi-monitor-build" do |pm|
  end

  config.vm.provider "vmware_desktop" do |v|
    v.vmx["memsize"] = "2048"
    v.vmx["numvcpus"] = "2"
    v.vmx["displayname"] = "pi-monitor-build"
  end

  # Provision the VM with everything we need to build Pi-Monitor
  config.vm.provision "shell", inline: <<-SHELL
    set -e

    echo "=== Updating package lists ==="
    apt-get update -qq

    # musl-tools: provides musl-gcc, the musl C compiler wrapper
    # musl-dev: provides musl header files and static libraries
    echo "=== Installing musl tools ==="
    apt-get install -y -qq musl-tools musl-dev build-essential

    # Install Rust via rustup for the vagrant user (not root)
    echo "=== Installing Rust toolchain ==="
    su - vagrant -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

    # This tells Rust: "I want to be able to compile for aarch64-unknown-linux-musl"
    echo "=== Adding musl target ==="
    su - vagrant -c 'source ~/.cargo/env && rustup target add aarch64-unknown-linux-musl'

    echo "=== Verifying installation ==="
    su - vagrant -c 'source ~/.cargo/env && rustc --version'
    su - vagrant -c 'source ~/.cargo/env && cargo --version'
    su - vagrant -c 'source ~/.cargo/env && rustup target list --installed'

    echo ""
    echo "=== Pi-Monitor build VM ready! ==="
    echo "Run: cd /vagrant && ./scripts/build.sh"
  SHELL

  config.vm.network "forwarded_port", guest: 9100, host: 9100
end