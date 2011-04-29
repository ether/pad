Summary: Etherpad
Name: etherpad
Version: 20110428
Release: 1
Group: Development/Tools
License: Apache License 2.0
Source: etherpad-20110428.tar.gz
URL: http://etherpad.org/
Packager: Norman Maul <nmaul@mozilla.com>
BuildArch: noarch
Buildroot: %{_tmppath}/%{name}-%{version}-root
Requires: scala, mysql-connector-java, java, screen, openoffice.org-core, openoffice.org-ure, openoffice.org-headless
Requires(pre): shadow-utils

%description
RPM package of the Etherpad web app. Includes custom hacked-up startup script and settings in /etc/sysconfig/etherpad.

%pre
getent group etherpad >/dev/null || groupadd -r etherpad
getent passwd etherpad >/dev/null || \
    useradd -r -g etherpad -d /usr/share/etherpad -s /sbin/nologin \
    -c "Etherpad system account" etherpad
exit 0

%prep
%setup

%build

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/share/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/init.d
mkdir -p $RPM_BUILD_ROOT/etc/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/sysconfig
mkdir -p $RPM_BUILD_ROOT/var/run/etherpad
cp $RPM_BUILD_DIR/%{name}-%{version}/redhat/etherpad.init-alt $RPM_BUILD_ROOT/etc/init.d/etherpad
cp $RPM_BUILD_DIR/%{name}-%{version}/redhat/etherpad.sysconfig $RPM_BUILD_ROOT/etc/sysconfig/etherpad
cp $RPM_BUILD_DIR/%{name}-%{version}/etherpad/etc/etherpad.localdev-default.properties $RPM_BUILD_ROOT/etc/etherpad/etherpad.local.properties
mv $RPM_BUILD_DIR/%{name}-%{version}/* $RPM_BUILD_ROOT/usr/share/etherpad

%files
%defattr(-,root,root)
/var/run/etherpad
%attr(755, etherpad, root) /usr/share/etherpad
/etc/init.d/etherpad
%config /etc/sysconfig/etherpad
%config /etc/etherpad/etherpad.local.properties


%clean
rm -rf $RPM_BUILD_ROOT
rm -rf $RPM_BUILD_DIR/%{name}-%{version}
