Summary: Etherpad
Name: etherpad
Version: 20110421
Release: 1
Group: Development/Tools
License: Apache License 2.0
Source0: etherpad-20110421.tar.gz
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
cp -a /usr/src/redhat/SOURCES/init.etherpad .
cp -a /usr/src/redhat/SOURCES/sysconfig.etherpad .
cp -a /usr/src/redhat/SOURCES/etherpad.local.properties .

%build

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/local/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/init.d
mkdir -p $RPM_BUILD_ROOT/etc/etherpad
mkdir -p $RPM_BUILD_ROOT/etc/sysconfig
mv $RPM_BUILD_DIR/%{name}-%{version}/init.etherpad $RPM_BUILD_ROOT/etc/init.d/etherpad
mv $RPM_BUILD_DIR/%{name}-%{version}/sysconfig.etherpad $RPM_BUILD_ROOT/etc/sysconfig/etherpad
mv $RPM_BUILD_DIR/%{name}-%{version}/etherpad.local.properties $RPM_BUILD_ROOT/etc/etherpad/
mv $RPM_BUILD_DIR/%{name}-%{version}/* $RPM_BUILD_ROOT/usr/local/etherpad

%files
%defattr(-,root,root)
/usr/local/etherpad/*
/etc/init.d/etherpad
%config /etc/sysconfig/etherpad
%config /etc/etherpad/etherpad.local.properties


%clean
rm -rf $RPM_BUILD_ROOT
rm -rf $RPM_BUILD_DIR/%{name}-%{version}
