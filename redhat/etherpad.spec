Summary: Etherpad
Name: etherpad
Version: 20110428
Release: 1
Group: Development/Tools
License: Apache License 2.0
Source0: etherpad-20110428.tar.gz
Source1: init.etherpad
Source2: sysconfig.etherpad
Source3: etherpad.local.properties
URL: http://etherpad.org/
Packager: Norman Maul <nmaul@mozilla.com>
BuildArch: noarch
Buildroot: %{_tmppath}/%{name}-%{version}-root
Requires: scala, mysql-connector-java, java, screen

%description
RPM package of the Etherpad web app. Includes custom hacked-up startup script and settings in /etc/sysconfig/etherpad.

%prep
%setup

%build

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/share/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/init.d
mkdir -p $RPM_BUILD_ROOT/etc/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/sysconfig
cp $RPM_BUILD_DIR/%{name}-%{version}/redhat/init.etherpad $RPM_BUILD_ROOT/etc/init.d/etherpad
cp $RPM_BUILD_DIR/%{name}-%{version}/redhat/etherpad.sysconfig $RPM_BUILD_ROOT/etc/sysconfig/etherpad
cp $RPM_BUILD_DIR/%{name}-%{version}/etherpad/etc/etherpad.localdev-default.properties $RPM_BUILD_ROOT/etc/etherpad/etherpad.local.properties
mv $RPM_BUILD_DIR/%{name}-%{version}/* $RPM_BUILD_ROOT/usr/share/etherpad

%files
%defattr(-,root,root)
/usr/share/etherpad
/usr/share/etherpad/*
/etc/init.d/etherpad
%config /etc/sysconfig/etherpad
%config /etc/etherpad/etherpad.local.properties


%clean
rm -rf $RPM_BUILD_ROOT
rm -rf $RPM_BUILD_DIR/%{name}-%{version}
