# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: Apache-2.0
# shellcheck shell=bash
# Assemble netevd user tarball (GitHub release / local pack).
#
# Usage: package_netevd_client_bundle STAGE BUILD_DIR VERSION

package_netevd_client_bundle() {
    local stage="$1" build_dir="$2" version="$3"
    local binary="${NETEVD_BINARY:-${build_dir}/target/release/netevd}"

    rm -rf "${stage}"
    mkdir -p "${stage}"

    if [[ ! -x "${binary}" ]]; then
        echo "package_netevd_client_bundle: missing executable ${binary}" >&2
        return 1
    fi

    cp "${binary}" "${stage}/netevd"
    chmod +x "${stage}/netevd"
    local lib="${build_dir}/scripts/lib"
    chmod +x "${lib}/copy-zyvor-legal-to-bundle.sh"
    "${lib}/copy-zyvor-legal-to-bundle.sh" "${stage}" "${build_dir}"

    cp "${build_dir}/systemd/netevd.service" "${stage}/netevd.service"
    cp "${build_dir}/config/netevd.example.yaml" "${stage}/config.example.yaml"

    cat > "${stage}/netevd.env.example" <<'ENV_EOF'
# Optional — copy to /etc/netevd/netevd.yaml after install
# NETEVD_LOG=info
ENV_EOF

    cat > "${stage}/install.sh" <<'INSTALL_EOF'
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
SUDO=""
[[ "$(id -u)" -ne 0 ]] && command -v sudo &>/dev/null && SUDO=sudo
$SUDO install -Dm755 "${ROOT}/netevd" /usr/bin/netevd
$SUDO install -Dm644 "${ROOT}/netevd.service" /lib/systemd/system/netevd.service
$SUDO install -Dm644 "${ROOT}/config.example.yaml" /etc/netevd/netevd.yaml
$SUDO useradd -r -M -s /usr/sbin/nologin netevd 2>/dev/null || true
$SUDO mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}
$SUDO systemctl daemon-reload
echo "Installed netevd — start with: sudo systemctl enable --now netevd"
INSTALL_EOF
    chmod +x "${stage}/install.sh"

    cat > "${stage}/uninstall.sh" <<'UNINSTALL_EOF'
#!/usr/bin/env bash
set -euo pipefail
SUDO=""
[[ "$(id -u)" -ne 0 ]] && command -v sudo &>/dev/null && SUDO=sudo
$SUDO systemctl stop netevd 2>/dev/null || true
$SUDO systemctl disable netevd 2>/dev/null || true
$SUDO rm -f /usr/bin/netevd /lib/systemd/system/netevd.service
$SUDO systemctl daemon-reload
echo "netevd removed (config under /etc/netevd not deleted)"
UNINSTALL_EOF
    chmod +x "${stage}/uninstall.sh"

    cat > "${stage}/QUICKSTART.txt" <<'QEOF'
netevd — install guide
======================

1. tar xzf netevd-*-linux-amd64.tar.gz && cd netevd-*-linux-amd64
2. sudo ./install.sh
3. sudo systemctl enable --now netevd
4. netevd --version

Config: /etc/netevd/netevd.yaml
Remove: sudo ./uninstall.sh

Enterprise (fleet / SLA): https://zyvor.dev/contact?utm_source=package&utm_medium=netevd
Demo: https://zyvor.dev/demo?utm_source=package&utm_medium=netevd
Sales: sales@zyvor.dev
QEOF

    cat > "${stage}/README.txt" <<README_EOF
netevd ${version} — Linux amd64 client bundle
==============================================

FILES
  LICENSE, LEGAL-INDEX.txt, docs/legal/
  netevd              Main daemon binary
  netevd.service      systemd unit
  config.example.yaml Sample configuration
  install.sh          Install to /usr/bin + systemd
  uninstall.sh        Remove binary and unit

REQUIREMENTS
  Linux with netlink, systemd-networkd / NetworkManager / dhclient

INSTALL
  tar xzf netevd-*-linux-amd64.tar.gz
  cd netevd-*-linux-amd64
  sudo ./install.sh
  sudo systemctl enable --now netevd

ENTERPRISE
  Production / SLA: https://zyvor.dev/contact?utm_source=package&utm_medium=netevd
  sales@zyvor.dev
README_EOF

    local req
    for req in install.sh uninstall.sh README.txt QUICKSTART.txt netevd netevd.service config.example.yaml \
        LICENSE LEGAL-INDEX.txt; do
        if [[ ! -e "${stage}/${req}" ]]; then
            echo "package_netevd_client_bundle: bundle missing ${req}" >&2
            return 1
        fi
    done

    echo "User bundle OK"
}

package_netevd_client_tarball() {
    local out_dir="$1" artifact="$2" stage="$3"
    mkdir -p "${out_dir}"
    (
        cd "${out_dir}"
        tar czf "${artifact}.tar.gz" "$(basename "${stage}")"
        sha256sum "${artifact}.tar.gz" | tee "${artifact}.tar.gz.sha256"
    )
}
