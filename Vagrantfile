# # -*- mode: ruby -*-
# # vi: set ft=ruby :

# Vagrant.configure("2") do |config|
#     config.vm.provider "docker" do |d|
#       d.image = "ubuntu:22.04"  # Use the image you just built
#       d.has_ssh = true          # Ensure SSH is enabled
#       d.cmd = ["/usr/sbin/sshd", "-D"]  # Run SSH
#     end
  
#     # Provisioning steps (optional but recommended to ensure proper setup)
#     config.vm.provision "shell", inline: <<-SHELL
#       apt-get update
#       apt-get install -y openssh-server
#       mkdir -p /var/run/sshd
#       echo 'vagrant:vagrant' | chpasswd
#       sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config
#       service ssh start
#     SHELL
#   end
  
Vagrant.configure("2") do |config|
      # Add these SSH settings
  config.ssh.username = "vagrant"
  config.ssh.password = "vagrant"
  config.ssh.insert_key = false
  config.vm.synced_folder "../", "/vagrant"

    config.vm.provider "docker" do |d|
    #   d.image = "ubuntu:22.04"
      d.remains_running = true
      d.has_ssh = true
      
      # This installs SSH and other needed packages before starting sshd
      d.build_dir = "."
      d.volumes = ["/var/run/docker.sock:/var/run/docker.sock"]

    end
  
    # Basic VM configuration
    config.vm.hostname = "ubuntu-docker"
  end