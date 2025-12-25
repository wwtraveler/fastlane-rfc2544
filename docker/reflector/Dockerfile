# Multi-threaded reflector — rockylinux:10 with SELinux compatibility
# Uses socat (multi-threaded) for 400G+ packet handling
FROM rockylinux/rockylinux:10
WORKDIR /fastlane
RUN dnf install -y iproute ethtool socat && dnf clean all
COPY docker/reflector/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
