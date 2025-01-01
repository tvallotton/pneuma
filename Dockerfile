FROM rust
RUN apt-get update
RUN apt-get install valgrind openssh-server -y

# Setup rust tooling
RUN rustup override set nightly
RUN rustup component add rustfmt clippy
RUN cargo install cargo-show-asm cargo-watch cargo-llvm-cov

# Setup ssh server
RUN mkdir /var/run/sshd
RUN echo 'root:root' | chpasswd
RUN mkdir /root/.ssh/authorized_keys
RUN chown root:root -R /root/.ssh && \
    chmod 600 /root/.ssh/authorized_keys
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config

EXPOSE 22

WORKDIR /home/app
COPY . .
CMD ["/usr/sbin/sshd", "-D"]
