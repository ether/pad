Summary: The Scala Programming Language
Name: scala
Version: 2.7.7.final
Release: 1
Group: Development/Tools
License: SCALA LICENSE (BSD-like)
Source: http://www.scala-lang.org/sites/default/files/linuxsoft_archives/downloads/distrib/files/scala-2.7.7.final.tgz
URL: http://www.scala-lang.org/
Packager: Norman Maul <nmaul@mozilla.com>
BuildArch: noarch
Buildroot: %{_tmppath}/%{name}-%{version}-root

%description
RPM package of the Scala programming language.


%prep
%setup

%build

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/local/scala
cp -a $RPM_BUILD_DIR/%{name}-%{version}/* $RPM_BUILD_ROOT/usr/local/scala

%files
%defattr(-,root,root)
/usr/local/scala/*

%clean
rm -rf $RPM_BUILD_ROOT
rm -rf $RPM_BUILD_DIR/%{name}-%{version}
