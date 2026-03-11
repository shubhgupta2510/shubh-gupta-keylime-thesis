# # Use an official Ubuntu as a parent image
# FROM ubuntu:22.04

# # Set environment variables
# ENV DEBIAN_FRONTEND=noninteractive

# # Update and install necessary packages
# RUN apt-get update && \
#     apt-get install -y \
#     openssh-server \
#     sudo \
#     curl \
#     lsb-release \
#     && rm -rf /var/lib/apt/lists/*

# # Create SSH directory
# RUN mkdir /var/run/sshd

# # Create a 'vagrant' user, set password, and add to sudo group
# RUN useradd -m -s /bin/bash vagrant && \
#     echo 'vagrant:vagrant' | chpasswd && \
#     adduser vagrant sudo

# # Enable root login via SSH
# RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

# # Expose the SSH port
# EXPOSE 22

# # Start the SSH service
# CMD ["/usr/sbin/sshd", "-D"]

FROM ubuntu:22.04
# Base packages and SSH setup
RUN apt-get update && apt-get install -y \
    docker.io \
    openssh-server \
    sudo \
    git \
    curl \
    wget \
    python3 \
    python3-pip \
    python3-setuptools \
    python3-venv \
    python3-tornado \
    python3-m2crypto \
    python3-cryptography \
    build-essential \
    gcc \
    make \
    iproute2 \
    net-tools \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install Docker Compose (latest v2.x)
RUN curl -SL https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64 -o /usr/local/bin/docker-compose && \
    chmod +x /usr/local/bin/docker-compose && \
    ln -s /usr/local/bin/docker-compose /usr/bin/docker-compose

# Verify Docker + Compose installation
RUN docker --version && docker-compose --version

RUN mkdir -p /var/run/sshd

# Create vagrant user properly
RUN useradd -m -s /bin/bash vagrant
RUN echo 'vagrant:vagrant' | chpasswd
RUN echo 'vagrant ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/vagrant
RUN chmod 0440 /etc/sudoers.d/vagrant

# Configure SSH correctly
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config
RUN sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config
RUN sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config

# Set proper permissions for the vagrant user's home
RUN mkdir -p /home/vagrant/.ssh
RUN chmod 700 /home/vagrant/.ssh
RUN chown -R vagrant:vagrant /home/vagrant

# Clone your thesis repo as vagrant user
USER vagrant
WORKDIR /home/vagrant
RUN git clone https://github.com/shubhgupta2510/shubh-gupta-keylime-thesis.git

USER root
EXPOSE 22
CMD ["/usr/sbin/sshd", "-D"]