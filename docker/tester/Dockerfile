FROM rockylinux/rockylinux:10
RUN dnf install -y libelf libbpf iproute ethtool \
    && dnf clean all
WORKDIR /fastlane
COPY target/release/fastlane /usr/local/bin/
COPY examples/ /fastlane/examples/
ENTRYPOINT ["fastlane"]
CMD ["--help"]
