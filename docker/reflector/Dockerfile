FROM rockylinux/rockylinux:10
WORKDIR /fastlane
RUN dnf install -y iproute ethtool httpd-tools \
    && dnf clean all
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
