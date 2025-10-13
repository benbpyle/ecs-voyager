Name:           ecs-voyager
Version:        0.2.7
Release:        1%{?dist}
Summary:        Terminal User Interface for AWS ECS Management

License:        MIT
URL:            https://github.com/benbpyle/ecs-voyager
Source0:        https://github.com/benbpyle/ecs-voyager/releases/download/v%{version}/ecs-voyager-v%{version}-x86_64-unknown-linux-gnu.tar.gz

BuildArch:      x86_64
Requires:       glibc >= 2.31

%description
ECS Voyager is a powerful Terminal User Interface (TUI) for managing
AWS ECS (Elastic Container Service) clusters, services, and tasks.

Features:
 * Interactive TUI with beautiful, responsive interface
 * Cluster, service, and task management
 * CloudWatch logs and metrics integration
 * ECS Exec for running commands in containers
 * Port forwarding to container ports
 * Search and filtering with regex support
 * Profile and region switching

%prep
%setup -q -n ecs-voyager-v%{version}-x86_64-unknown-linux-gnu

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 ecs-voyager %{buildroot}%{_bindir}/ecs-voyager

%files
%{_bindir}/ecs-voyager

%post
echo ""
echo "ECS Voyager has been installed!"
echo ""
echo "Usage: ecs-voyager"
echo "       ecs-voyager --help"
echo ""
echo "For ECS Exec and Port Forwarding features, install session-manager-plugin:"
echo "  See: https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html"
echo ""

%changelog
* Sun Jan 12 2025 benbpyle <benbpyle@users.noreply.github.com> - 0.2.7-1
- Search & Filtering Release
- Regex pattern support
- Service and task status filters
- Launch type filtering
- Multi-criteria filtering

* Sat Jan 11 2025 benbpyle <benbpyle@users.noreply.github.com> - 0.2.6-1
- Initial RPM package release
