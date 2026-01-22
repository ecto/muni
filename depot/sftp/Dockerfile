# Simple SFTP server for session uploads
FROM alpine:3.21

RUN apk add --no-cache openssh-server

# Create sftp-only user
RUN adduser -D -s /sbin/nologin bvr && \
    echo "bvr:dev" | chpasswd && \
    mkdir -p /home/bvr/sessions && \
    chown bvr:bvr /home/bvr/sessions

# Configure sshd for SFTP only
RUN mkdir -p /etc/ssh && \
    ssh-keygen -A && \
    echo "PasswordAuthentication yes" >> /etc/ssh/sshd_config && \
    echo "PermitRootLogin no" >> /etc/ssh/sshd_config && \
    echo "Subsystem sftp internal-sftp" >> /etc/ssh/sshd_config && \
    echo "Match User bvr" >> /etc/ssh/sshd_config && \
    echo "  ChrootDirectory /home/bvr" >> /etc/ssh/sshd_config && \
    echo "  ForceCommand internal-sftp" >> /etc/ssh/sshd_config && \
    echo "  AllowTcpForwarding no" >> /etc/ssh/sshd_config

# Fix permissions for chroot
RUN chown root:root /home/bvr && chmod 755 /home/bvr

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D", "-e"]
