#
# Enkrnett OpenWrt package. The daemon is not compiled here: it is built outside the SDK by
# openwrt/build-mips.sh with the pinned nightly and this SDK's toolchain (ENK-BUILD-007, 009),
# so that the same bytes go into the package and into the measurement. This Makefile installs
# the prebuilt daemon, the UI files, and everything else Enkrnett adds (ENK-FOOT-003).
#
#   ENKRNETT_PREBUILT=/path/with/enkrnettd/and/ui make package/enkrnett/compile
#
include $(TOPDIR)/rules.mk

PKG_NAME:=enkrnett
PKG_VERSION:=0.1.0
PKG_RELEASE:=1
PKG_LICENSE:=Apache-2.0
PKG_LICENSE_FILES:=LICENSE
PKG_MAINTAINER:=nabbisen

include $(INCLUDE_DIR)/package.mk

define Package/enkrnett
  SECTION:=net
  CATEGORY:=Network
  TITLE:=Enkrnett: guest Wi-Fi with OWE for people without IT training
  URL:=https://github.com/nabbisen/enkrnett
  DEPENDS:=+wpad-basic-mbedtls +dnsmasq +firewall4
  USERID:=enkrnett=1900:enkrnett=1900
endef

define Package/enkrnett/description
 A small daemon and a phone-first settings screen that give a venue a guest
 Wi-Fi network encrypted per visitor (OWE), separated from the venue's own
 network, managed through a dedicated management Wi-Fi. Specification:
 docs/src/specification/ in the repository.
endef

define Package/enkrnett/conffiles
/etc/config/enkrnett
/etc/enkrnett/
endef

ENKRNETT_PREBUILT?=$(error ENKRNETT_PREBUILT must name the directory that holds the prebuilt enkrnettd and ui/)

define Build/Prepare
	mkdir -p $(PKG_BUILD_DIR)
endef

define Build/Configure
endef

define Build/Compile
endef

define Package/enkrnett/install
	$(INSTALL_DIR) $(1)/usr/sbin
	$(INSTALL_BIN) $(ENKRNETT_PREBUILT)/enkrnettd $(1)/usr/sbin/enkrnettd
	$(INSTALL_DIR) $(1)/usr/share/enkrnett/ui
	$(CP) $(ENKRNETT_PREBUILT)/ui/. $(1)/usr/share/enkrnett/ui/
	$(INSTALL_DIR) $(1)/usr/share/acl.d
	$(INSTALL_DATA) ./files/enkrnett.acl.json $(1)/usr/share/acl.d/enkrnett.json
	$(INSTALL_DIR) $(1)/etc/capabilities
	$(INSTALL_DATA) ./files/enkrnett.capabilities.json $(1)/etc/capabilities/enkrnett.json
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/enkrnett.init $(1)/etc/init.d/enkrnett
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) ./files/enkrnett.config $(1)/etc/config/enkrnett
	$(INSTALL_DIR) $(1)/etc/uci-defaults
	$(INSTALL_BIN) ./files/enkrnett.defaults $(1)/etc/uci-defaults/90-enkrnett
	$(INSTALL_DIR) $(1)/etc/enkrnett
endef

$(eval $(call BuildPackage,enkrnett))
