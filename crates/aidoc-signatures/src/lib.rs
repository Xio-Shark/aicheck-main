use std::collections::HashSet;

use aidoc_core::{Category, EnvironmentSnapshot, RuleHit, ToolVersion};

#[derive(Clone)]
struct Rule {
    id: &'static str,
    category: Category,
    title: &'static str,
    patterns: &'static [&'static str],
    verify_commands: &'static [&'static str],
    cause_en: &'static str,
    cause_zh: &'static str,
    fix_suggestions_en: &'static [&'static str],
    fix_suggestions_zh: &'static [&'static str],
}

const EMPTY_HINTS: &[&str] = &[];

static RULES: &[Rule] = &[
    Rule {
        id: "python_not_found",
        category: Category::Path,
        title: "Python executable not found",
        patterns: &["python: command not found", "'python' is not recognized"],
        verify_commands: &["which python", "where python"],
        cause_en: "Python is not on PATH or not installed in current shell.",
        cause_zh: "当前 Shell 中 Python 未安装或未进入 PATH。",
        fix_suggestions_en: &[
            "Install Python 3.x and reopen terminal/IDE.",
            "Ensure python executable directory is in PATH.",
            "Configure the IDE interpreter to the installed Python path.",
        ],
        fix_suggestions_zh: &[
            "安装 Python 3.x 后重启终端与 IDE。",
            "确保 python 可执行目录已加入 PATH。",
            "在 IDE 中将解释器显式设置为已安装路径。",
        ],
    },
    Rule {
        id: "pip_not_found",
        category: Category::Path,
        title: "pip executable not found",
        patterns: &["pip: command not found", "'pip' is not recognized"],
        verify_commands: &["python -m pip -V", "which pip", "where pip"],
        cause_en: "pip entrypoint is missing or points to a removed interpreter.",
        cause_zh: "pip 入口缺失，或关联解释器已失效。",
        fix_suggestions_en: &[
            "Use `python -m ensurepip --upgrade` to bootstrap pip.",
            "Use `python -m pip` instead of bare `pip` in scripts.",
        ],
        fix_suggestions_zh: &[
            "使用 `python -m ensurepip --upgrade` 初始化 pip。",
            "脚本中优先使用 `python -m pip`，避免依赖裸 `pip`。",
        ],
    },
    Rule {
        id: "npm_not_found",
        category: Category::Path,
        title: "npm executable not found",
        patterns: &["bash: npm: command not found", "'npm' is not recognized"],
        verify_commands: &["node -v", "which npm", "where npm"],
        cause_en: "Node.js runtime or npm shim is not available in PATH.",
        cause_zh: "Node.js 或 npm shim 未进入 PATH。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "gcc_not_found",
        category: Category::Toolchain,
        title: "C compiler not found",
        patterns: &["gcc: command not found", "'gcc' is not recognized"],
        verify_commands: &["gcc --version", "clang --version", "cc --version"],
        cause_en: "C compiler is missing or not available in current PATH.",
        cause_zh: "C 编译器缺失，或当前 PATH 中不可用。",
        fix_suggestions_en: &[
            "Install build-essential (Linux) or Build Tools/MSVC (Windows).",
            "Reopen IDE after installation so PATH is refreshed.",
        ],
        fix_suggestions_zh: &[
            "安装 build-essential（Linux）或 Build Tools/MSVC（Windows）。",
            "安装后重启 IDE，使 PATH 生效。",
        ],
    },
    Rule {
        id: "gpp_not_found",
        category: Category::Toolchain,
        title: "C++ compiler not found",
        patterns: &["g++: command not found", "'g++' is not recognized"],
        verify_commands: &["g++ --version", "clang++ --version", "c++ --version"],
        cause_en: "C++ compiler is missing or not available in current PATH.",
        cause_zh: "C++ 编译器缺失，或当前 PATH 中不可用。",
        fix_suggestions_en: &[
            "Install g++/clang++ and make sure compiler bin is in PATH.",
            "Configure C/C++ extension compilerPath explicitly in IDE settings.",
        ],
        fix_suggestions_zh: &[
            "安装 g++/clang++ 并确保编译器 bin 目录进入 PATH。",
            "在 IDE 设置中显式配置 C/C++ 编译器路径。",
        ],
    },
    Rule {
        id: "python_header_missing",
        category: Category::Dependency,
        title: "Python development headers missing",
        patterns: &[
            "python.h: no such file or directory",
            "cannot open include file: 'python.h'",
        ],
        verify_commands: &[
            "python -c \"import sysconfig; print(sysconfig.get_config_var('INCLUDEPY'))\"",
        ],
        cause_en: "Python runtime exists but development headers are missing.",
        cause_zh: "已安装 Python 运行时，但缺少开发头文件。",
        fix_suggestions_en: &[
            "Install python-dev/python3-dev package.",
            "Use the same Python interpreter for build and IDE.",
        ],
        fix_suggestions_zh: &[
            "安装 python-dev 或 python3-dev 包。",
            "确保构建与 IDE 使用同一解释器。",
        ],
    },
    Rule {
        id: "cpp_standard_headers_missing",
        category: Category::Dependency,
        title: "C++ standard headers missing in compiler environment",
        patterns: &[
            "fatal error: iostream: no such file or directory",
            "cannot open source file \"iostream\"",
        ],
        verify_commands: &["g++ -v", "clang++ -v"],
        cause_en: "Compiler toolchain or include search path is incomplete.",
        cause_zh: "编译器工具链或头文件搜索路径不完整。",
        fix_suggestions_en: &[
            "Install full C++ toolchain (g++ or clang++ with standard library).",
            "Check IDE includePath and compilerPath configuration.",
        ],
        fix_suggestions_zh: &[
            "安装完整 C++ 工具链（g++ 或 clang++ 与标准库）。",
            "检查 IDE 的 includePath 与 compilerPath 配置。",
        ],
    },
    Rule {
        id: "cmake_compiler_not_set",
        category: Category::Version,
        title: "CMake cannot find C/C++ compiler",
        patterns: &[
            "cmake_c_compiler not set",
            "cmake_cxx_compiler not set",
            "no c compiler could be found",
        ],
        verify_commands: &["cmake --version", "echo $CC", "echo $CXX"],
        cause_en: "CMake cannot resolve compiler from PATH or CC/CXX.",
        cause_zh: "CMake 无法从 PATH 或 CC/CXX 解析编译器。",
        fix_suggestions_en: &[
            "Set CC/CXX explicitly to valid compiler binaries.",
            "Run CMake from a shell where compiler PATH is initialized.",
        ],
        fix_suggestions_zh: &[
            "将 CC/CXX 显式设置为有效编译器路径。",
            "在已初始化编译器 PATH 的终端执行 CMake。",
        ],
    },
    Rule {
        id: "ide_interpreter_not_selected",
        category: Category::Path,
        title: "IDE interpreter or compiler profile is not configured",
        patterns: &[
            "no python interpreter selected",
            "python interpreter is invalid",
            "unable to resolve configuration with compilerpath",
        ],
        verify_commands: &["which python", "where python", "which g++", "where g++"],
        cause_en: "IDE process is using a different environment than terminal.",
        cause_zh: "IDE 进程使用的环境与终端不一致。",
        fix_suggestions_en: &[
            "Configure interpreter/compiler path in IDE settings explicitly.",
            "Restart IDE from a terminal that already has the right PATH.",
        ],
        fix_suggestions_zh: &[
            "在 IDE 设置中显式配置解释器/编译器路径。",
            "从已具备正确 PATH 的终端启动 IDE。",
        ],
    },
    Rule {
        id: "certificate_verify_failed",
        category: Category::Cert,
        title: "TLS certificate verification failed",
        patterns: &["certificate_verify_failed", "x509", "ssl: certificate"],
        verify_commands: &["curl -I https://pypi.org", "curl -I https://github.com"],
        cause_en: "Local trust store or TLS interception causes certificate validation failure.",
        cause_zh: "本地证书信任链异常或被代理劫持导致 TLS 校验失败。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "proxy_connection_refused",
        category: Category::Proxy,
        title: "Proxy endpoint refused connection",
        patterns: &["proxy error", "connection refused", "proxyconnect"],
        verify_commands: &["echo $HTTP_PROXY", "netsh winhttp show proxy"],
        cause_en: "Proxy endpoint is unreachable or stale in environment/config.",
        cause_zh: "代理地址不可达或配置残留。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "permission_denied",
        category: Category::Permission,
        title: "Permission denied",
        patterns: &["permission denied", "eacces", "operation not permitted"],
        verify_commands: &["id", "whoami /all", "ls -ld ."],
        cause_en: "Current user lacks required permissions for target path or action.",
        cause_zh: "当前用户对目标路径或动作权限不足。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "git_auth_failed",
        category: Category::Dependency,
        title: "Git authentication failed",
        patterns: &[
            "authentication failed",
            "repository not found",
            "fatal: could not read",
        ],
        verify_commands: &["git config --list --show-origin", "git remote -v"],
        cause_en: "Git credentials are missing, expired, or mismatched with remote URL.",
        cause_zh: "Git 凭据缺失、过期或与远端地址不匹配。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "docker_daemon_unreachable",
        category: Category::Dependency,
        title: "Docker daemon unreachable",
        patterns: &[
            "cannot connect to the docker daemon",
            "is the docker daemon running",
        ],
        verify_commands: &["docker version", "docker info"],
        cause_en: "Docker daemon is stopped or current user cannot access docker socket.",
        cause_zh: "Docker daemon 未运行，或当前用户无权访问 docker socket。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "cargo_linker_missing",
        category: Category::Toolchain,
        title: "Rust linker/toolchain issue",
        patterns: &[
            "linker",
            "ld: cannot find",
            "failed to run custom build command",
        ],
        verify_commands: &["rustc -V", "cargo -V", "which cc", "ldd --version"],
        cause_en: "Required linker/build dependency is absent or misconfigured.",
        cause_zh: "链接器或构建依赖缺失，或工具链配置错误。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "java_home_invalid",
        category: Category::Version,
        title: "JAVA_HOME invalid",
        patterns: &["java_home", "no java runtime present"],
        verify_commands: &["java -version", "javac -version", "echo $JAVA_HOME"],
        cause_en: "JAVA_HOME points to invalid location or JDK tools are missing.",
        cause_zh: "JAVA_HOME 指向无效目录，或 JDK 工具缺失。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "wsl_path_mismatch",
        category: Category::Wsl,
        title: "WSL path mismatch",
        patterns: &["/mnt/c/", "wsl", "windows path"],
        verify_commands: &["uname -a", "echo $PATH", "which python"],
        cause_en: "Windows and Linux path/toolchain contexts are mixed.",
        cause_zh: "Windows 与 Linux 的路径或工具链上下文混用。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "dns_resolution_failed",
        category: Category::Network,
        title: "DNS resolution failed",
        patterns: &[
            "temporary failure in name resolution",
            "could not resolve host",
            "name or service not known",
        ],
        verify_commands: &[
            "resolvectl status",
            "nslookup github.com",
            "curl -I https://github.com",
        ],
        cause_en: "DNS resolver is unavailable or incorrectly configured.",
        cause_zh: "DNS 解析器不可用或配置错误。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "node_not_found",
        category: Category::Path,
        title: "Node.js executable not found",
        patterns: &["node: command not found", "'node' is not recognized"],
        verify_commands: &["node -v", "which node", "where node"],
        cause_en: "Node.js runtime is not installed or not in PATH.",
        cause_zh: "Node.js 运行时未安装，或未加入 PATH。",
        fix_suggestions_en: &[
            "Install Node.js LTS and reopen terminal/IDE.",
            "Ensure Node.js install directory is included in PATH.",
        ],
        fix_suggestions_zh: &[
            "安装 Node.js LTS 后重启终端/IDE。",
            "确保 Node.js 安装目录已加入 PATH。",
        ],
    },
    Rule {
        id: "git_not_found",
        category: Category::Path,
        title: "Git executable not found",
        patterns: &["git: command not found", "'git' is not recognized"],
        verify_commands: &["git --version", "which git", "where git"],
        cause_en: "Git client is not installed or unavailable in PATH.",
        cause_zh: "Git 客户端未安装，或当前 PATH 不可用。",
        fix_suggestions_en: &[
            "Install Git and reopen terminal.",
            "Verify PATH includes git executable directory.",
        ],
        fix_suggestions_zh: &["安装 Git 后重启终端。", "确认 PATH 包含 git 可执行目录。"],
    },
    Rule {
        id: "python_module_not_found",
        category: Category::Dependency,
        title: "Python module missing",
        patterns: &["modulenotfounderror: no module named"],
        verify_commands: &[
            "python -m pip list",
            "python -c \"import site; print(site.getsitepackages())\"",
        ],
        cause_en:
            "Requested Python package is not installed in the active interpreter environment.",
        cause_zh: "当前解释器环境中缺少所需 Python 包。",
        fix_suggestions_en: &[
            "Install dependency in the same interpreter: python -m pip install <package>.",
            "Ensure IDE and terminal use the same virtual environment.",
        ],
        fix_suggestions_zh: &[
            "在同一解释器中安装依赖：python -m pip install <package>。",
            "确保 IDE 与终端使用同一虚拟环境。",
        ],
    },
    Rule {
        id: "pip_externally_managed",
        category: Category::Permission,
        title: "Python environment is externally managed",
        patterns: &[
            "externally-managed-environment",
            "this environment is externally managed",
        ],
        verify_commands: &["python -m pip -V", "python -m venv .venv"],
        cause_en:
            "System Python is protected by distribution policy and blocks global pip install.",
        cause_zh: "系统 Python 受发行版策略保护，禁止全局 pip 安装。",
        fix_suggestions_en: &[
            "Create and use a virtual environment for package installation.",
            "Use distribution package manager if system-wide install is required.",
        ],
        fix_suggestions_zh: &[
            "创建并使用虚拟环境安装依赖。",
            "若必须全局安装，请使用发行版包管理器。",
        ],
    },
    Rule {
        id: "pip_ssl_module_missing",
        category: Category::Cert,
        title: "Python SSL module unavailable",
        patterns: &["ssl module in python is not available"],
        verify_commands: &["python -c \"import ssl; print(ssl.OPENSSL_VERSION)\""],
        cause_en: "Python build lacks SSL support, causing TLS package download failures.",
        cause_zh: "Python 构建缺少 SSL 支持，导致 TLS 下载失败。",
        fix_suggestions_en: &[
            "Use a Python build with OpenSSL support.",
            "Reinstall Python from official package source.",
        ],
        fix_suggestions_zh: &[
            "使用带 OpenSSL 支持的 Python 构建。",
            "从官方来源重新安装 Python。",
        ],
    },
    Rule {
        id: "pip_wheel_build_failed",
        category: Category::Toolchain,
        title: "pip wheel build failed",
        patterns: &[
            "failed building wheel for",
            "error: subprocess-exited-with-error",
        ],
        verify_commands: &[
            "python -m pip --version",
            "gcc --version",
            "python -m pip debug --verbose",
        ],
        cause_en: "Native build prerequisites are missing for one or more Python packages.",
        cause_zh: "构建某些 Python 包所需的本地编译依赖缺失。",
        fix_suggestions_en: &[
            "Install compiler toolchain and Python development headers.",
            "Prefer prebuilt wheels compatible with current platform.",
        ],
        fix_suggestions_zh: &[
            "安装编译工具链与 Python 开发头文件。",
            "优先使用与当前平台兼容的预编译 wheel。",
        ],
    },
    Rule {
        id: "npm_dependency_conflict",
        category: Category::Dependency,
        title: "npm dependency tree conflict",
        patterns: &[
            "npm err! code eresolve",
            "unable to resolve dependency tree",
        ],
        verify_commands: &["npm -v", "npm ls --depth=0"],
        cause_en: "Declared dependency versions are mutually incompatible.",
        cause_zh: "声明的依赖版本之间存在不兼容约束。",
        fix_suggestions_en: &[
            "Align peer dependency versions in package.json.",
            "Regenerate lockfile after updating conflicting dependencies.",
        ],
        fix_suggestions_zh: &[
            "在 package.json 中对齐 peer 依赖版本。",
            "更新冲突依赖后重新生成 lockfile。",
        ],
    },
    Rule {
        id: "npm_permission_denied",
        category: Category::Permission,
        title: "npm permission denied",
        patterns: &["npm err! code eacces", "npm err! syscall"],
        verify_commands: &["npm config get prefix", "ls -ld ~/.npm"],
        cause_en: "Current user lacks write permission to npm cache/prefix directory.",
        cause_zh: "当前用户对 npm 缓存或 prefix 目录无写权限。",
        fix_suggestions_en: &[
            "Use a user-writable npm prefix or node version manager.",
            "Avoid global installs into protected system directories.",
        ],
        fix_suggestions_zh: &[
            "将 npm prefix 配置到用户可写目录或使用版本管理器。",
            "避免向受保护系统目录执行全局安装。",
        ],
    },
    Rule {
        id: "npm_registry_timeout",
        category: Category::Network,
        title: "npm registry request timed out",
        patterns: &["npm err! code etimedout", "network timeout at"],
        verify_commands: &[
            "npm config get registry",
            "curl -I https://registry.npmjs.org",
        ],
        cause_en: "Network path to npm registry is unstable or blocked by proxy/firewall.",
        cause_zh: "到 npm registry 的网络链路不稳定，或被代理/防火墙拦截。",
        fix_suggestions_en: &[
            "Verify proxy and DNS settings for current shell.",
            "Retry after confirming registry reachability with curl HEAD.",
        ],
        fix_suggestions_zh: &[
            "检查当前 shell 的代理与 DNS 设置。",
            "先用 curl HEAD 确认可达，再重试安装。",
        ],
    },
    Rule {
        id: "npm_registry_not_found",
        category: Category::Dependency,
        title: "npm package not found in registry",
        patterns: &["npm err! 404 not found", "is not in this registry"],
        verify_commands: &["npm view <package> version", "npm config get registry"],
        cause_en: "Package name/version is incorrect or not available in configured registry.",
        cause_zh: "包名/版本错误，或当前 registry 中不存在该包。",
        fix_suggestions_en: &[
            "Verify package name and version in package.json.",
            "Check whether private registry requires authentication.",
        ],
        fix_suggestions_zh: &[
            "核对 package.json 中的包名与版本。",
            "确认私有 registry 是否需要登录认证。",
        ],
    },
    Rule {
        id: "npm_package_json_missing",
        category: Category::Path,
        title: "package.json missing",
        patterns: &["npm err! enoent", "could not read package.json"],
        verify_commands: &["pwd", "ls -la", "cat package.json"],
        cause_en: "Current working directory is not a Node project root.",
        cause_zh: "当前工作目录不是 Node 项目根目录。",
        fix_suggestions_en: &[
            "Run npm commands in directory that contains package.json.",
            "Ensure CI checkout path matches expected project root.",
        ],
        fix_suggestions_zh: &[
            "在包含 package.json 的目录中执行 npm 命令。",
            "确认 CI 检出路径与项目根目录一致。",
        ],
    },
    Rule {
        id: "git_ssl_certificate_error",
        category: Category::Cert,
        title: "Git SSL certificate verification error",
        patterns: &[
            "ssl certificate problem",
            "server certificate verification failed",
        ],
        verify_commands: &[
            "git config --list --show-origin",
            "curl -I https://github.com",
        ],
        cause_en: "TLS trust chain is invalid for current Git HTTPS requests.",
        cause_zh: "当前 Git HTTPS 请求的 TLS 信任链无效。",
        fix_suggestions_en: &[
            "Import required CA certificate into trust store.",
            "Avoid disabling SSL verify globally.",
        ],
        fix_suggestions_zh: &["将所需 CA 证书导入系统信任链。", "避免全局关闭 SSL 校验。"],
    },
    Rule {
        id: "git_index_lock_exists",
        category: Category::Dependency,
        title: "Git index lock exists",
        patterns: &["another git process seems to be running", ".git/index.lock"],
        verify_commands: &["ps -ef | rg git", "ls -l .git/index.lock"],
        cause_en: "Previous Git operation terminated unexpectedly and left a lock file.",
        cause_zh: "上一次 Git 操作异常结束，残留锁文件。",
        fix_suggestions_en: &[
            "Ensure no active git process is running.",
            "Remove stale lock only after confirming no running git process.",
        ],
        fix_suggestions_zh: &["先确认无活跃 git 进程。", "确认后再清理残留锁文件。"],
    },
    Rule {
        id: "docker_compose_not_found",
        category: Category::Path,
        title: "Docker Compose command unavailable",
        patterns: &[
            "docker: 'compose' is not a docker command",
            "docker-compose: command not found",
        ],
        verify_commands: &["docker compose version", "docker --help"],
        cause_en:
            "Docker Compose plugin is not installed or not discoverable by current Docker CLI.",
        cause_zh: "Docker Compose 插件未安装，或当前 Docker CLI 无法发现该插件。",
        fix_suggestions_en: &[
            "Install Docker Compose v2 plugin compatible with current Docker CLI.",
            "Verify Docker CLI plugin path configuration.",
        ],
        fix_suggestions_zh: &[
            "安装与当前 Docker CLI 兼容的 Docker Compose v2 插件。",
            "检查 Docker CLI 插件路径配置。",
        ],
    },
    Rule {
        id: "docker_socket_permission_denied",
        category: Category::Permission,
        title: "No permission to access Docker socket",
        patterns: &[
            "permission denied while trying to connect to the docker daemon socket",
            "docker daemon socket",
        ],
        verify_commands: &["id", "ls -l /var/run/docker.sock", "groups"],
        cause_en: "Current user is not allowed to access Docker daemon socket.",
        cause_zh: "当前用户无权限访问 Docker daemon socket。",
        fix_suggestions_en: &[
            "Use a user account with docker group membership.",
            "Re-login session after group membership changes.",
        ],
        fix_suggestions_zh: &[
            "使用属于 docker 组的用户运行命令。",
            "变更组权限后重新登录会话。",
        ],
    },
    Rule {
        id: "docker_no_space_left",
        category: Category::Dependency,
        title: "Docker storage full",
        patterns: &["no space left on device"],
        verify_commands: &["docker system df", "df -h"],
        cause_en: "Disk space is insufficient for image/layer extraction or build cache.",
        cause_zh: "磁盘空间不足，无法完成镜像层解压或构建缓存写入。",
        fix_suggestions_en: &[
            "Prune unused images and build cache.",
            "Increase disk allocation for Docker data root.",
        ],
        fix_suggestions_zh: &[
            "清理未使用镜像和构建缓存。",
            "扩大 Docker 数据目录可用磁盘容量。",
        ],
    },
    Rule {
        id: "javac_not_found",
        category: Category::Path,
        title: "JDK compiler not found",
        patterns: &["javac: command not found", "'javac' is not recognized"],
        verify_commands: &["java -version", "javac -version", "echo $JAVA_HOME"],
        cause_en: "JRE exists but JDK tools are missing from PATH.",
        cause_zh: "仅有 JRE 或 PATH 缺少 JDK 工具。",
        fix_suggestions_en: &[
            "Install JDK and ensure javac is in PATH.",
            "Set JAVA_HOME to JDK root instead of JRE path.",
        ],
        fix_suggestions_zh: &[
            "安装 JDK 并确保 PATH 包含 javac。",
            "将 JAVA_HOME 指向 JDK 根目录而非 JRE。",
        ],
    },
    Rule {
        id: "java_class_version_mismatch",
        category: Category::Version,
        title: "Java class version mismatch",
        patterns: &[
            "unsupportedclassversionerror",
            "has been compiled by a more recent version",
        ],
        verify_commands: &["java -version", "javac -version"],
        cause_en: "Runtime JRE version is lower than bytecode target version.",
        cause_zh: "运行时 JRE 版本低于字节码目标版本。",
        fix_suggestions_en: &[
            "Align runtime and build JDK major versions.",
            "Rebuild artifact with target version compatible with runtime.",
        ],
        fix_suggestions_zh: &[
            "统一运行时与构建时 JDK 主版本。",
            "按运行环境支持版本重新构建产物。",
        ],
    },
    Rule {
        id: "maven_not_found",
        category: Category::Path,
        title: "Maven executable not found",
        patterns: &["mvn: command not found", "'mvn' is not recognized"],
        verify_commands: &["mvn -v", "which mvn", "where mvn"],
        cause_en: "Maven is not installed or not present in PATH.",
        cause_zh: "Maven 未安装，或未加入 PATH。",
        fix_suggestions_en: &[
            "Install Maven and reopen terminal.",
            "Verify Maven bin directory is configured in PATH.",
        ],
        fix_suggestions_zh: &[
            "安装 Maven 后重启终端。",
            "确认 Maven bin 目录已加入 PATH。",
        ],
    },
    Rule {
        id: "cargo_not_found",
        category: Category::Path,
        title: "Cargo executable not found",
        patterns: &["cargo: command not found", "'cargo' is not recognized"],
        verify_commands: &["cargo -V", "which cargo", "where cargo"],
        cause_en: "Rust toolchain is not installed or cargo bin path is missing.",
        cause_zh: "Rust 工具链未安装，或 cargo bin 路径缺失。",
        fix_suggestions_en: &[
            "Install Rust via rustup and reopen shell.",
            "Ensure ~/.cargo/bin is present in PATH.",
        ],
        fix_suggestions_zh: &[
            "通过 rustup 安装 Rust 并重启 shell。",
            "确保 PATH 包含 ~/.cargo/bin。",
        ],
    },
    Rule {
        id: "rustc_not_found",
        category: Category::Path,
        title: "rustc executable not found",
        patterns: &["rustc: command not found", "'rustc' is not recognized"],
        verify_commands: &["rustc -V", "which rustc", "where rustc"],
        cause_en: "Rust compiler is unavailable in current environment.",
        cause_zh: "当前环境中 Rust 编译器不可用。",
        fix_suggestions_en: &[
            "Install rustup toolchain and verify rustc in PATH.",
            "Use the same shell profile where rustup env is loaded.",
        ],
        fix_suggestions_zh: &[
            "安装 rustup 工具链并确认 PATH 可访问 rustc。",
            "使用已加载 rustup 环境的 shell 配置。",
        ],
    },
    Rule {
        id: "cargo_target_missing",
        category: Category::Dependency,
        title: "Rust target not installed",
        patterns: &[
            "target may not be installed",
            "consider downloading the target with `rustup target add`",
        ],
        verify_commands: &["rustup target list --installed", "rustc -Vv"],
        cause_en: "Requested Rust compilation target is not installed locally.",
        cause_zh: "请求的 Rust 编译目标在本地未安装。",
        fix_suggestions_en: &[
            "Install missing target with rustup target add <triple>.",
            "Ensure build scripts use installed target triple.",
        ],
        fix_suggestions_zh: &[
            "使用 rustup target add <triple> 安装缺失目标。",
            "确保构建脚本使用已安装的目标三元组。",
        ],
    },
    Rule {
        id: "cargo_pkg_config_missing",
        category: Category::Toolchain,
        title: "pkg-config missing for native dependency build",
        patterns: &["pkg-config command could not be found"],
        verify_commands: &[
            "pkg-config --version",
            "which pkg-config",
            "where pkg-config",
        ],
        cause_en: "Native crate build requires pkg-config but executable is missing.",
        cause_zh: "本地依赖构建需要 pkg-config，但系统缺少该工具。",
        fix_suggestions_en: &[
            "Install pkg-config and reopen terminal.",
            "Verify PKG_CONFIG_PATH if custom library locations are used.",
        ],
        fix_suggestions_zh: &[
            "安装 pkg-config 后重启终端。",
            "若使用自定义库路径，请检查 PKG_CONFIG_PATH。",
        ],
    },
    Rule {
        id: "cargo_openssl_missing",
        category: Category::Dependency,
        title: "OpenSSL development files missing",
        patterns: &[
            "could not find directory of openssl installation",
            "openssl-sys",
        ],
        verify_commands: &["openssl version -a", "pkg-config --libs openssl"],
        cause_en: "OpenSSL headers/libs are missing or not discoverable for build.",
        cause_zh: "构建所需 OpenSSL 头文件/库缺失，或无法被发现。",
        fix_suggestions_en: &[
            "Install OpenSSL development package and pkg-config metadata.",
            "Set OPENSSL_DIR when using non-default installation path.",
        ],
        fix_suggestions_zh: &[
            "安装 OpenSSL 开发包及 pkg-config 元数据。",
            "若使用非默认路径，请设置 OPENSSL_DIR。",
        ],
    },
    Rule {
        id: "go_not_found",
        category: Category::Path,
        title: "Go executable not found",
        patterns: &["go: command not found", "'go' is not recognized"],
        verify_commands: &["go version", "which go", "where go"],
        cause_en: "Go toolchain is not installed or not in PATH.",
        cause_zh: "Go 工具链未安装，或当前 PATH 不可用。",
        fix_suggestions_en: &[
            "Install Go and reopen terminal/IDE.",
            "Ensure go binary directory is included in PATH.",
        ],
        fix_suggestions_zh: &["安装 Go 后重启终端/IDE。", "确保 PATH 包含 go 可执行目录。"],
    },
    Rule {
        id: "go_checksum_mismatch",
        category: Category::Dependency,
        title: "Go module checksum mismatch",
        patterns: &["checksum mismatch", "go.sum"],
        verify_commands: &["go env GOPROXY", "go mod verify"],
        cause_en: "Downloaded module checksum differs from expected go.sum entry.",
        cause_zh: "下载模块校验和与 go.sum 记录不一致。",
        fix_suggestions_en: &[
            "Verify module source/proxy and integrity before updating checksums.",
            "Use trusted GOPROXY and clean corrupted module cache.",
        ],
        fix_suggestions_zh: &[
            "更新校验前先确认模块来源与完整性。",
            "使用可信 GOPROXY 并清理损坏模块缓存。",
        ],
    },
    Rule {
        id: "pip_hash_mismatch",
        category: Category::Dependency,
        title: "pip hash verification mismatch",
        patterns: &[
            "these packages do not match the hashes",
            "hashes are required in --require-hashes mode",
        ],
        verify_commands: &[
            "python -m pip --version",
            "python -m pip install --require-hashes -r requirements.txt",
        ],
        cause_en: "Downloaded package hash does not match the pinned requirement hash.",
        cause_zh: "下载包哈希与 requirements 锁定值不一致。",
        fix_suggestions_en: &[
            "Verify package source before updating requirement hashes.",
            "Use trusted index/proxy for dependency download.",
        ],
        fix_suggestions_zh: &[
            "更新哈希前先确认包来源可信。",
            "使用可信镜像或代理下载依赖。",
        ],
    },
    Rule {
        id: "pip_no_matching_distribution",
        category: Category::Version,
        title: "No matching distribution for package",
        patterns: &["no matching distribution found for"],
        verify_commands: &["python --version", "python -m pip index versions <package>"],
        cause_en: "Requested package version is incompatible with current Python/platform.",
        cause_zh: "请求的包版本与当前 Python 或平台不兼容。",
        fix_suggestions_en: &[
            "Select a package version compatible with current runtime.",
            "Upgrade Python when package requires newer interpreter.",
        ],
        fix_suggestions_zh: &[
            "选择与当前运行时兼容的包版本。",
            "若包要求更高版本，请升级 Python。",
        ],
    },
    Rule {
        id: "pip_requires_python_mismatch",
        category: Category::Version,
        title: "Package requires different Python version",
        patterns: &["require a different python version", "requires-python"],
        verify_commands: &["python --version", "python -m pip debug --verbose"],
        cause_en: "Current interpreter version is outside package Requires-Python range.",
        cause_zh: "当前解释器版本不满足包的 Requires-Python 约束。",
        fix_suggestions_en: &[
            "Switch interpreter to compatible major/minor version.",
            "Pin package to a release that supports current Python.",
        ],
        fix_suggestions_zh: &[
            "切换到兼容的 Python 主次版本。",
            "将依赖固定到支持当前 Python 的版本。",
        ],
    },
    Rule {
        id: "pip_proxy_auth_required",
        category: Category::Proxy,
        title: "Proxy requires authentication for pip",
        patterns: &["407 proxy authentication required", "proxyerror"],
        verify_commands: &["echo $HTTP_PROXY", "python -m pip config list"],
        cause_en: "Proxy is configured but credentials are missing/invalid.",
        cause_zh: "已配置代理，但认证凭据缺失或无效。",
        fix_suggestions_en: &[
            "Set valid proxy credentials in environment or pip config.",
            "Test proxy reachability with curl HEAD first.",
        ],
        fix_suggestions_zh: &[
            "在环境变量或 pip 配置中设置有效代理凭据。",
            "先用 curl HEAD 验证代理可达性。",
        ],
    },
    Rule {
        id: "pip_index_connection_failed",
        category: Category::Network,
        title: "pip cannot connect to package index",
        patterns: &[
            "httpsconnectionpool(host='pypi.org'",
            "max retries exceeded with url",
        ],
        verify_commands: &["curl -I https://pypi.org", "python -m pip config list"],
        cause_en: "Network path to package index is unstable or blocked.",
        cause_zh: "到包索引站点的网络路径不稳定或被拦截。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "venv_activation_missing",
        category: Category::Path,
        title: "Virtual environment activation script missing",
        patterns: &[
            "venv/bin/activate: no such file or directory",
            ".venv/scripts/activate is not recognized",
        ],
        verify_commands: &["pwd", "ls -la", "python -m venv .venv"],
        cause_en: "Current directory does not contain expected virtual environment.",
        cause_zh: "当前目录缺少预期的虚拟环境目录。",
        fix_suggestions_en: &[
            "Create virtual environment in project root before activation.",
            "Run activation command with path that matches current shell.",
        ],
        fix_suggestions_zh: &[
            "在项目根目录先创建虚拟环境再激活。",
            "使用与当前 shell 匹配的激活脚本路径。",
        ],
    },
    Rule {
        id: "node_engine_incompatible",
        category: Category::Version,
        title: "Node engine version incompatible",
        patterns: &[
            "the engine \"node\" is incompatible with this module",
            "expected version",
        ],
        verify_commands: &["node -v", "npm config get engine-strict"],
        cause_en: "Current Node.js version does not satisfy package engines constraint.",
        cause_zh: "当前 Node.js 版本不满足包的 engines 约束。",
        fix_suggestions_en: &[
            "Switch to project-required Node LTS version.",
            "Align CI and local Node version strategy.",
        ],
        fix_suggestions_zh: &[
            "切换到项目要求的 Node LTS 版本。",
            "统一 CI 与本地 Node 版本策略。",
        ],
    },
    Rule {
        id: "npm_ci_lock_missing",
        category: Category::Dependency,
        title: "npm ci requires lockfile",
        patterns: &["npm ci can only install with an existing package-lock.json"],
        verify_commands: &["ls -la", "npm -v"],
        cause_en: "npm ci requires package-lock.json but lockfile is missing.",
        cause_zh: "npm ci 需要 package-lock.json，但当前缺失锁文件。",
        fix_suggestions_en: &[
            "Commit package-lock.json to repository.",
            "Use npm install only when lockfile generation is intended.",
        ],
        fix_suggestions_zh: &[
            "将 package-lock.json 提交到仓库。",
            "仅在需要生成锁文件时使用 npm install。",
        ],
    },
    Rule {
        id: "npm_integrity_checksum_failed",
        category: Category::Dependency,
        title: "npm tarball integrity check failed",
        patterns: &["integrity checksum failed when using sha512"],
        verify_commands: &["npm cache verify", "npm config get registry"],
        cause_en: "Downloaded tarball integrity does not match lockfile metadata.",
        cause_zh: "下载 tarball 的完整性与锁文件元数据不一致。",
        fix_suggestions_en: &[
            "Verify registry mirror integrity and clear corrupted cache.",
            "Regenerate lockfile only after dependency source is confirmed.",
        ],
        fix_suggestions_zh: &[
            "验证镜像源完整性并清理损坏缓存。",
            "确认依赖来源后再重建锁文件。",
        ],
    },
    Rule {
        id: "npm_auth_e401",
        category: Category::Permission,
        title: "npm authentication required (E401)",
        patterns: &[
            "npm err! code e401",
            "unable to authenticate, need: basic realm",
        ],
        verify_commands: &["npm whoami", "npm config get registry"],
        cause_en: "Registry requires authentication token but current auth is invalid.",
        cause_zh: "仓库需要认证令牌，但当前认证信息无效。",
        fix_suggestions_en: &[
            "Refresh npm token and verify registry scope mapping.",
            "Avoid storing plain credentials in project files.",
        ],
        fix_suggestions_zh: &[
            "更新 npm 令牌并核对 registry scope 映射。",
            "避免在项目文件中保存明文凭据。",
        ],
    },
    Rule {
        id: "npm_tar_unpack_error",
        category: Category::Dependency,
        title: "npm tar archive unpack error",
        patterns: &["npm err! tar_bad_archive", "unrecognized archive format"],
        verify_commands: &["npm cache verify", "npm config get registry"],
        cause_en: "Package archive is corrupted in transit or cache.",
        cause_zh: "包归档在传输或缓存中损坏。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "yarn_not_found",
        category: Category::Path,
        title: "Yarn executable not found",
        patterns: &["yarn: command not found", "'yarn' is not recognized"],
        verify_commands: &["yarn -v", "corepack --version"],
        cause_en: "Yarn package manager is not installed or not in PATH.",
        cause_zh: "Yarn 包管理器未安装或未进入 PATH。",
        fix_suggestions_en: &[
            "Enable corepack or install yarn globally.",
            "Reopen terminal after package manager installation.",
        ],
        fix_suggestions_zh: &["启用 corepack 或全局安装 yarn。", "安装完成后重启终端。"],
    },
    Rule {
        id: "pnpm_not_found",
        category: Category::Path,
        title: "pnpm executable not found",
        patterns: &["pnpm: command not found", "'pnpm' is not recognized"],
        verify_commands: &["pnpm -v", "corepack --version"],
        cause_en: "pnpm package manager is not installed or not in PATH.",
        cause_zh: "pnpm 包管理器未安装或未进入 PATH。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "git_non_fast_forward",
        category: Category::Dependency,
        title: "Git push rejected by non-fast-forward",
        patterns: &["non-fast-forward", "[rejected]"],
        verify_commands: &["git status -sb", "git log --oneline --decorate -5"],
        cause_en: "Local branch history is behind remote or diverged.",
        cause_zh: "本地分支历史落后于远端或已分叉。",
        fix_suggestions_en: &[
            "Integrate remote updates before pushing.",
            "Use rebase/merge strategy defined by project workflow.",
        ],
        fix_suggestions_zh: &[
            "先整合远端更新再推送。",
            "按项目规范选择 rebase 或 merge 策略。",
        ],
    },
    Rule {
        id: "git_merge_conflict",
        category: Category::Dependency,
        title: "Git merge conflict requires manual resolution",
        patterns: &["automatic merge failed; fix conflicts and then commit the result"],
        verify_commands: &["git status", "git diff --name-only --diff-filter=U"],
        cause_en: "Auto-merge failed due to conflicting changes in same files.",
        cause_zh: "同一文件存在冲突修改，自动合并失败。",
        fix_suggestions_en: &[
            "Resolve conflict markers and run tests before commit.",
            "Keep conflict resolution aligned with target branch behavior.",
        ],
        fix_suggestions_zh: &[
            "处理冲突标记后执行测试再提交。",
            "冲突解决结果需与目标分支行为一致。",
        ],
    },
    Rule {
        id: "git_detached_head",
        category: Category::Dependency,
        title: "Git operation in detached HEAD",
        patterns: &["fatal: you are not currently on a branch"],
        verify_commands: &["git status -sb", "git branch --show-current"],
        cause_en: "Repository is in detached HEAD state and lacks active branch context.",
        cause_zh: "仓库处于 detached HEAD 状态，缺少活动分支上下文。",
        fix_suggestions_en: &[
            "Create/switch to a branch before push operations.",
            "Avoid committing directly on detached commit refs.",
        ],
        fix_suggestions_zh: &[
            "推送前先创建或切换到分支。",
            "避免在 detached 提交上直接开发。",
        ],
    },
    Rule {
        id: "git_host_key_verification_failed",
        category: Category::Cert,
        title: "SSH host key verification failed",
        patterns: &["host key verification failed"],
        verify_commands: &["ssh -T git@github.com", "ssh-keygen -F github.com"],
        cause_en: "Known hosts entry is missing or mismatched for remote host key.",
        cause_zh: "known_hosts 条目缺失或与远端主机密钥不匹配。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "git_lfs_not_installed",
        category: Category::Dependency,
        title: "Git LFS executable missing",
        patterns: &["git-lfs filter-process: git-lfs: command not found"],
        verify_commands: &["git lfs version", "git config --show-origin -l | rg lfs"],
        cause_en: "Repository expects Git LFS but local environment lacks git-lfs.",
        cause_zh: "仓库依赖 Git LFS，但本地缺少 git-lfs。",
        fix_suggestions_en: &[
            "Install Git LFS and run git lfs install in current user context.",
            "Re-clone or re-fetch LFS objects after installation.",
        ],
        fix_suggestions_zh: &[
            "安装 Git LFS 并在当前用户下执行 git lfs install。",
            "安装后重新拉取 LFS 对象。",
        ],
    },
    Rule {
        id: "git_shallow_update_rejected",
        category: Category::Dependency,
        title: "Shallow repository update rejected",
        patterns: &["shallow update not allowed"],
        verify_commands: &["git rev-parse --is-shallow-repository", "git remote -v"],
        cause_en: "Remote update requires full history while local clone is shallow.",
        cause_zh: "远端更新需要完整历史，但当前克隆为浅克隆。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "docker_build_context_forbidden",
        category: Category::Path,
        title: "Docker build context path is invalid",
        patterns: &["forbidden path outside the build context"],
        verify_commands: &["pwd", "docker build -f Dockerfile ."],
        cause_en: "Dockerfile COPY/ADD references files outside build context.",
        cause_zh: "Dockerfile 的 COPY/ADD 引用了构建上下文之外的路径。",
        fix_suggestions_en: &[
            "Move required files into build context directory.",
            "Adjust docker build context to project root.",
        ],
        fix_suggestions_zh: &[
            "将所需文件移动到构建上下文目录。",
            "将 docker build 上下文调整到项目根目录。",
        ],
    },
    Rule {
        id: "dockerfile_parse_error",
        category: Category::Dependency,
        title: "Dockerfile parse syntax error",
        patterns: &["dockerfile parse error line"],
        verify_commands: &["docker build --no-cache .", "sed -n '1,120p' Dockerfile"],
        cause_en: "Dockerfile contains invalid directive syntax.",
        cause_zh: "Dockerfile 包含非法指令语法。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "docker_manifest_unknown",
        category: Category::Dependency,
        title: "Image manifest not found",
        patterns: &["manifest unknown", "not found: manifest unknown"],
        verify_commands: &[
            "docker pull <image>:<tag>",
            "docker manifest inspect <image>:<tag>",
        ],
        cause_en: "Requested image tag does not exist in target registry.",
        cause_zh: "请求的镜像标签在目标仓库不存在。",
        fix_suggestions_en: &[
            "Verify image tag spelling and architecture availability.",
            "Pin to an existing published image tag.",
        ],
        fix_suggestions_zh: &[
            "核对镜像标签拼写与架构可用性。",
            "固定为仓库已发布的有效标签。",
        ],
    },
    Rule {
        id: "docker_network_not_found",
        category: Category::Network,
        title: "Docker network not found",
        patterns: &["network not found"],
        verify_commands: &["docker network ls", "docker compose config"],
        cause_en: "Referenced Docker network does not exist in current context.",
        cause_zh: "当前上下文不存在被引用的 Docker 网络。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "docker_buildx_not_found",
        category: Category::Path,
        title: "Docker buildx plugin unavailable",
        patterns: &["docker: 'buildx' is not a docker command"],
        verify_commands: &["docker buildx version", "docker --help"],
        cause_en: "Docker buildx plugin is missing or not discoverable by CLI.",
        cause_zh: "Docker buildx 插件缺失或 CLI 无法发现。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "docker_login_required",
        category: Category::Permission,
        title: "Registry requires docker login",
        patterns: &["pull access denied for", "may require 'docker login'"],
        verify_commands: &["docker info", "docker login <registry>"],
        cause_en: "Registry denies pull due to missing or invalid credentials.",
        cause_zh: "仓库因认证缺失或无效拒绝镜像拉取。",
        fix_suggestions_en: &[
            "Authenticate against the target registry.",
            "Verify image repository and namespace permissions.",
        ],
        fix_suggestions_zh: &["先对目标仓库执行登录认证。", "确认镜像仓库与命名空间权限。"],
    },
    Rule {
        id: "gradle_not_found",
        category: Category::Path,
        title: "Gradle executable not found",
        patterns: &["gradle: command not found", "'gradle' is not recognized"],
        verify_commands: &["gradle -v", "which gradle", "where gradle"],
        cause_en: "Gradle CLI is not installed or not present in PATH.",
        cause_zh: "Gradle CLI 未安装或未进入 PATH。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "maven_java_home_not_defined",
        category: Category::Version,
        title: "Maven JAVA_HOME misconfigured",
        patterns: &[
            "the java_home environment variable is not defined correctly",
            "maven",
        ],
        verify_commands: &["echo $JAVA_HOME", "mvn -v", "java -version"],
        cause_en: "JAVA_HOME does not point to a valid JDK location for Maven.",
        cause_zh: "JAVA_HOME 未指向 Maven 可用的有效 JDK 路径。",
        fix_suggestions_en: &[
            "Set JAVA_HOME to JDK root and reopen terminal.",
            "Avoid pointing JAVA_HOME to JRE-only installation.",
        ],
        fix_suggestions_zh: &[
            "将 JAVA_HOME 设置为 JDK 根目录并重启终端。",
            "避免将 JAVA_HOME 指向仅含 JRE 的路径。",
        ],
    },
    Rule {
        id: "maven_dependency_resolution_failed",
        category: Category::Dependency,
        title: "Maven dependency resolution failed",
        patterns: &["could not resolve dependencies for project"],
        verify_commands: &["mvn -U -X dependency:tree", "echo $HTTP_PROXY"],
        cause_en: "One or more Maven dependencies cannot be fetched from repositories.",
        cause_zh: "一个或多个 Maven 依赖无法从仓库拉取。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "maven_invalid_target_release",
        category: Category::Version,
        title: "Maven compiler target release invalid",
        patterns: &["invalid target release"],
        verify_commands: &["mvn -v", "java -version", "javac -version"],
        cause_en: "Configured source/target level is unsupported by current JDK.",
        cause_zh: "配置的 source/target 级别不被当前 JDK 支持。",
        fix_suggestions_en: &[
            "Align maven-compiler-plugin release with installed JDK.",
            "Use toolchains plugin when multiple JDK versions coexist.",
        ],
        fix_suggestions_zh: &[
            "将 maven-compiler-plugin 的 release 与已安装 JDK 对齐。",
            "多 JDK 并存时使用 toolchains 插件显式指定。",
        ],
    },
    Rule {
        id: "rustup_not_found",
        category: Category::Path,
        title: "rustup executable not found",
        patterns: &["rustup: command not found", "'rustup' is not recognized"],
        verify_commands: &["rustup --version", "which rustup", "where rustup"],
        cause_en: "Rustup toolchain manager is unavailable in current PATH.",
        cause_zh: "当前 PATH 不可用 rustup 工具链管理器。",
        fix_suggestions_en: &[
            "Install rustup and source shell profile again.",
            "Ensure ~/.cargo/bin is exported in PATH.",
        ],
        fix_suggestions_zh: &[
            "安装 rustup 后重新加载 shell 配置。",
            "确保 PATH 导出 ~/.cargo/bin。",
        ],
    },
    Rule {
        id: "rust_toolchain_not_installed",
        category: Category::Dependency,
        title: "Requested Rust toolchain is not installed",
        patterns: &["toolchain 'stable-x86_64-unknown-linux-gnu' is not installed"],
        verify_commands: &["rustup toolchain list", "rustup show active-toolchain"],
        cause_en: "Build requests a Rust toolchain not available in local rustup.",
        cause_zh: "构建请求的 Rust 工具链在本地 rustup 中不存在。",
        fix_suggestions_en: &[
            "Install required toolchain with rustup toolchain install.",
            "Pin toolchain via rust-toolchain.toml for reproducibility.",
        ],
        fix_suggestions_zh: &[
            "使用 rustup toolchain install 安装所需工具链。",
            "通过 rust-toolchain.toml 固定工具链以保证可复现。",
        ],
    },
    Rule {
        id: "cargo_frozen_lockfile",
        category: Category::Dependency,
        title: "Cargo lockfile update blocked by --frozen",
        patterns: &["the lock file needs to be updated but --frozen was passed"],
        verify_commands: &["cargo check --locked", "git status --short Cargo.lock"],
        cause_en: "Cargo.lock is out of date for manifest changes under frozen mode.",
        cause_zh: "在 frozen 模式下，Cargo.lock 与清单变更不同步。",
        fix_suggestions_en: &[
            "Update lockfile in non-frozen mode and commit the result.",
            "Use --locked in CI after lockfile is synchronized.",
        ],
        fix_suggestions_zh: &[
            "先在非 frozen 模式更新锁文件并提交。",
            "锁文件同步后在 CI 使用 --locked。",
        ],
    },
    Rule {
        id: "cargo_registry_timeout",
        category: Category::Network,
        title: "Cargo registry download timeout",
        patterns: &[
            "failed to download from `https://index.crates.io`",
            "operation timed out",
        ],
        verify_commands: &["cargo fetch -v", "curl -I https://index.crates.io"],
        cause_en: "Network to crates index is unstable or blocked by proxy/firewall.",
        cause_zh: "到 crates 索引的网络不稳定或被代理/防火墙拦截。",
        fix_suggestions_en: EMPTY_HINTS,
        fix_suggestions_zh: EMPTY_HINTS,
    },
    Rule {
        id: "cargo_git_fetch_auth",
        category: Category::Dependency,
        title: "Cargo git dependency authentication failed",
        patterns: &[
            "authentication required but no callback set",
            "failed to fetch into",
        ],
        verify_commands: &["git config --list --show-origin", "ssh -T git@github.com"],
        cause_en: "Cargo cannot authenticate when fetching git-based dependencies.",
        cause_zh: "Cargo 拉取 git 依赖时认证失败。",
        fix_suggestions_en: &[
            "Configure git credentials/SSH keys for dependency source.",
            "Ensure non-interactive environment has required auth material.",
        ],
        fix_suggestions_zh: &[
            "为依赖源配置 git 凭据或 SSH 密钥。",
            "确保非交互环境具备所需认证材料。",
        ],
    },
    Rule {
        id: "go_required_module_missing",
        category: Category::Dependency,
        title: "Go required module missing",
        patterns: &["no required module provides package"],
        verify_commands: &["go env GOPROXY", "go mod tidy", "go list -m all"],
        cause_en: "Requested import path is missing from current module requirements.",
        cause_zh: "当前模块依赖中缺少对应导入路径。",
        fix_suggestions_en: &[
            "Add the missing module requirement and tidy dependencies.",
            "Verify import path spelling and module boundaries.",
        ],
        fix_suggestions_zh: &[
            "补充缺失模块依赖并整理 go.mod。",
            "核对导入路径拼写与模块边界。",
        ],
    },
    Rule {
        id: "systemd_not_booted",
        category: Category::Wsl,
        title: "systemd is not available in current environment",
        patterns: &[
            "system has not been booted with systemd",
            "failed to connect to bus: host is down",
        ],
        verify_commands: &["ps -p 1 -o comm=", "cat /proc/1/comm"],
        cause_en: "Current runtime is not booted with systemd (common in WSL/container).",
        cause_zh: "当前运行环境并非以 systemd 启动（WSL/容器常见）。",
        fix_suggestions_en: &[
            "Avoid relying on systemctl in non-systemd environments.",
            "Use runtime-specific daemon startup path (Docker Desktop/entrypoint/supervisor).",
        ],
        fix_suggestions_zh: &[
            "在非 systemd 环境避免依赖 systemctl。",
            "改用运行时对应的守护进程启动方式（Docker Desktop/entrypoint/supervisor）。",
        ],
    },
];

pub fn match_signatures(text: &str) -> Vec<RuleHit> {
    let lower = text.to_lowercase();
    let mut hits: Vec<RuleHit> = Vec::new();

    for rule in RULES {
        let mut hit_count = 0usize;
        for pattern in rule.patterns {
            if lower.contains(pattern) {
                hit_count += 1;
            }
        }

        if hit_count == 0 {
            continue;
        }

        let confidence = (0.65 + (hit_count as f32 - 1.0) * 0.12).clamp(0.65, 0.97);
        let evidence = extract_evidence(&lower, text, rule.patterns, 6);
        hits.push(to_rule_hit(rule, confidence, evidence));
    }

    hits.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    hits
}

pub fn detect_environment_issues(snapshot: &EnvironmentSnapshot) -> Vec<RuleHit> {
    let mut hits = Vec::new();
    let tool_names = collect_tool_names(&snapshot.toolchains);

    if !has_any_tool(&tool_names, &["python", "python3"]) {
        hits.push(build_env_hit(
            "dev_python_missing",
            Category::Toolchain,
            0.93,
            "Python runtime not detected",
            "Python executable is missing from current environment PATH.",
            "当前环境 PATH 未检测到 Python 可执行文件。",
            vec![
                format!("os={}", snapshot.os),
                format!("shell={:?}", snapshot.shell),
            ],
            vec![
                "python --version",
                "python3 --version",
                "which python",
                "where python",
            ],
            vec![
                "Install Python 3.x and reopen IDE/terminal.",
                "Set IDE interpreter to the installed Python executable.",
            ],
            vec![
                "安装 Python 3.x 后重启 IDE 与终端。",
                "在 IDE 中将解释器设置为已安装 Python 路径。",
            ],
        ));
    }

    if !has_any_tool(&tool_names, &["pip", "pip3"])
        && has_any_tool(&tool_names, &["python", "python3"])
    {
        hits.push(build_env_hit(
            "dev_pip_missing",
            Category::Dependency,
            0.77,
            "pip command not detected",
            "Python exists but pip entrypoint is missing from PATH.",
            "检测到 Python，但 PATH 中缺少 pip 入口。",
            vec!["python detected".to_string()],
            vec!["python -m ensurepip --upgrade", "python -m pip -V"],
            vec![
                "Bootstrap pip with python -m ensurepip --upgrade.",
                "Use python -m pip in build scripts to avoid PATH shim issues.",
            ],
            vec![
                "使用 python -m ensurepip --upgrade 初始化 pip。",
                "在构建脚本中优先使用 python -m pip，避免 PATH shim 问题。",
            ],
        ));
    }

    if !has_any_tool(&tool_names, &["gcc", "clang", "cc", "cl"]) {
        hits.push(build_env_hit(
            "dev_c_compiler_missing",
            Category::Toolchain,
            0.91,
            "C compiler not detected",
            "No C compiler (gcc/clang/cc/cl) found in current PATH.",
            "当前 PATH 未检测到 C 编译器（gcc/clang/cc/cl）。",
            vec![format!("toolchains={}", tool_names.len())],
            vec![
                "gcc --version",
                "clang --version",
                "cc --version",
                "where cl",
            ],
            vec![
                "Install build-essential or clang toolchain (Linux/macOS).",
                "Install Visual Studio Build Tools and Developer Command Prompt (Windows).",
            ],
            vec![
                "安装 build-essential 或 clang 工具链（Linux/macOS）。",
                "安装 Visual Studio Build Tools 并使用 Developer Command Prompt（Windows）。",
            ],
        ));
    }

    if !has_any_tool(&tool_names, &["g++", "clang++", "c++", "cl"]) {
        hits.push(build_env_hit(
            "dev_cpp_compiler_missing",
            Category::Toolchain,
            0.9,
            "C++ compiler not detected",
            "No C++ compiler (g++/clang++/c++/cl) found in current PATH.",
            "当前 PATH 未检测到 C++ 编译器（g++/clang++/c++/cl）。",
            vec![format!("toolchains={}", tool_names.len())],
            vec![
                "g++ --version",
                "clang++ --version",
                "c++ --version",
                "where cl",
            ],
            vec![
                "Install C++ compiler package and standard library headers.",
                "Set explicit compilerPath in IDE C/C++ extension settings.",
            ],
            vec![
                "安装 C++ 编译器与标准库头文件。",
                "在 IDE C/C++ 扩展中显式设置 compilerPath。",
            ],
        ));
    }

    if !has_any_tool(&tool_names, &["cmake", "make", "ninja"]) {
        hits.push(build_env_hit(
            "dev_build_tool_missing",
            Category::Dependency,
            0.79,
            "Build orchestrator not detected",
            "No cmake/make/ninja command found. Native builds may fail.",
            "未检测到 cmake/make/ninja，原生构建可能失败。",
            vec![format!("toolchains={}", tool_names.len())],
            vec!["cmake --version", "make --version", "ninja --version"],
            vec![
                "Install cmake and one build backend (make or ninja).",
                "Reopen IDE to refresh PATH after installation.",
            ],
            vec![
                "安装 cmake 与至少一种构建后端（make 或 ninja）。",
                "安装后重启 IDE 以刷新 PATH。",
            ],
        ));
    }

    if should_flag_ide_env_mismatch(snapshot) {
        hits.push(build_env_hit(
            "ide_environment_mismatch",
            Category::Path,
            0.82,
            "Terminal and IDE environment mismatch",
            "PATH content suggests IDE and shell may be using different runtime contexts.",
            "PATH 内容显示 IDE 与 Shell 可能处于不同运行时上下文。",
            snapshot.path_preview.iter().take(4).cloned().collect(),
            vec![
                "echo $PATH",
                "which python",
                "which gcc",
                "where python",
                "where cl",
            ],
            vec![
                "Start IDE from the same terminal where toolchain works.",
                "Set interpreter/compiler path explicitly in IDE settings.",
                "Avoid mixing WSL and Windows toolchain paths in one profile.",
            ],
            vec![
                "从已能识别工具链的同一终端启动 IDE。",
                "在 IDE 中显式配置解释器/编译器路径。",
                "避免在同一配置中混用 WSL 与 Windows 路径。",
            ],
        ));
    }

    hits.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    hits
}

fn to_rule_hit(rule: &Rule, confidence: f32, evidence: Vec<String>) -> RuleHit {
    RuleHit {
        id: rule.id.to_string(),
        category: rule.category.clone(),
        confidence,
        title: rule.title.to_string(),
        cause_en: rule.cause_en.to_string(),
        cause_zh: rule.cause_zh.to_string(),
        evidence,
        verify_commands: rule
            .verify_commands
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        fix_suggestions_en: rule
            .fix_suggestions_en
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        fix_suggestions_zh: rule
            .fix_suggestions_zh
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_env_hit(
    id: &str,
    category: Category,
    confidence: f32,
    title: &str,
    cause_en: &str,
    cause_zh: &str,
    evidence: Vec<String>,
    verify_commands: Vec<&str>,
    fix_suggestions_en: Vec<&str>,
    fix_suggestions_zh: Vec<&str>,
) -> RuleHit {
    RuleHit {
        id: id.to_string(),
        category,
        confidence,
        title: title.to_string(),
        cause_en: cause_en.to_string(),
        cause_zh: cause_zh.to_string(),
        evidence,
        verify_commands: verify_commands
            .iter()
            .map(|item| item.to_string())
            .collect(),
        fix_suggestions_en: fix_suggestions_en
            .iter()
            .map(|item| item.to_string())
            .collect(),
        fix_suggestions_zh: fix_suggestions_zh
            .iter()
            .map(|item| item.to_string())
            .collect(),
    }
}

fn collect_tool_names(tools: &[ToolVersion]) -> HashSet<String> {
    tools
        .iter()
        .map(|item| item.name.to_ascii_lowercase())
        .collect()
}

fn has_any_tool(tool_names: &HashSet<String>, names: &[&str]) -> bool {
    names.iter().any(|name| tool_names.contains(*name))
}

fn should_flag_ide_env_mismatch(snapshot: &EnvironmentSnapshot) -> bool {
    if snapshot.path_preview.is_empty() {
        return true;
    }

    let os = snapshot.os.to_ascii_lowercase();
    let has_windows_segment = snapshot.path_preview.iter().any(|segment| {
        let s = segment.to_ascii_lowercase();
        s.contains("\\windows\\") || s.contains("/mnt/c/windows")
    });
    let has_linux_segment = snapshot.path_preview.iter().any(|segment| {
        let s = segment.to_ascii_lowercase();
        s.contains("/usr/bin") || s.contains("/usr/local/bin")
    });

    if os == "windows" && has_linux_segment {
        return true;
    }
    if os == "linux" && has_windows_segment {
        return true;
    }
    false
}

fn extract_evidence(
    lower_text: &str,
    original_text: &str,
    patterns: &[&str],
    max_lines: usize,
) -> Vec<String> {
    let original_lines: Vec<&str> = original_text.lines().collect();
    let lower_lines: Vec<&str> = lower_text.lines().collect();

    let mut result = Vec::new();
    for (idx, line) in lower_lines.iter().enumerate() {
        if patterns.iter().any(|pattern| line.contains(pattern)) {
            if let Some(raw) = original_lines.get(idx) {
                result.push((*raw).to_string());
            }
            if result.len() >= max_lines {
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use aidoc_core::{EnvironmentSnapshot, ProxySnapshot, ToolVersion};

    use super::{detect_environment_issues, match_signatures};

    #[test]
    fn should_match_python_not_found() {
        let log = "bash: python: command not found";
        let hits = match_signatures(log);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].id, "python_not_found");
    }

    #[test]
    fn should_match_all_builtin_rules_with_fixtures() {
        let fixtures = vec![
            ("pip: command not found", "pip_not_found"),
            ("bash: npm: command not found", "npm_not_found"),
            ("gcc: command not found", "gcc_not_found"),
            ("g++: command not found", "gpp_not_found"),
            (
                "fatal error: Python.h: No such file or directory",
                "python_header_missing",
            ),
            (
                "fatal error: iostream: No such file or directory",
                "cpp_standard_headers_missing",
            ),
            ("CMAKE_C_COMPILER not set", "cmake_compiler_not_set"),
            ("No Python interpreter selected", "ide_interpreter_not_selected"),
            ("ssl: certificate verify failed", "certificate_verify_failed"),
            ("proxy error: connection refused", "proxy_connection_refused"),
            ("EACCES: permission denied", "permission_denied"),
            (
                "fatal: Authentication failed for https://example.com/repo.git",
                "git_auth_failed",
            ),
            (
                "Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?",
                "docker_daemon_unreachable",
            ),
            (
                "ld: cannot find -lssl, failed to run custom build command",
                "cargo_linker_missing",
            ),
            ("no java runtime present, JAVA_HOME is invalid", "java_home_invalid"),
            ("wsl windows path mismatch: /mnt/c/Windows", "wsl_path_mismatch"),
            (
                "curl: (6) Could not resolve host: github.com",
                "dns_resolution_failed",
            ),
            ("node: command not found", "node_not_found"),
            ("git: command not found", "git_not_found"),
            (
                "ModuleNotFoundError: No module named 'requests'",
                "python_module_not_found",
            ),
            (
                "error: externally-managed-environment",
                "pip_externally_managed",
            ),
            (
                "failed building wheel for cryptography",
                "pip_wheel_build_failed",
            ),
            (
                "npm ERR! code ERESOLVE\nunable to resolve dependency tree",
                "npm_dependency_conflict",
            ),
            (
                "npm ERR! code EACCES\nnpm ERR! syscall open",
                "npm_permission_denied",
            ),
            (
                "npm ERR! code ETIMEDOUT\nnetwork timeout at: https://registry.npmjs.org",
                "npm_registry_timeout",
            ),
            (
                "npm ERR! 404 Not Found - GET https://registry.npmjs.org/pkg - Not found",
                "npm_registry_not_found",
            ),
            (
                "npm ERR! enoent Could not read package.json",
                "npm_package_json_missing",
            ),
            (
                "fatal: unable to access 'https://example.com': SSL certificate problem: unable to get local issuer certificate",
                "git_ssl_certificate_error",
            ),
            (
                "fatal: Unable to create '.git/index.lock': File exists.\nanother git process seems to be running",
                "git_index_lock_exists",
            ),
            (
                "docker: 'compose' is not a docker command.",
                "docker_compose_not_found",
            ),
            (
                "Got permission denied while trying to connect to the Docker daemon socket",
                "docker_socket_permission_denied",
            ),
            ("no space left on device", "docker_no_space_left"),
            ("javac: command not found", "javac_not_found"),
            (
                "java.lang.UnsupportedClassVersionError: class has been compiled by a more recent version",
                "java_class_version_mismatch",
            ),
            ("mvn: command not found", "maven_not_found"),
            ("cargo: command not found", "cargo_not_found"),
            ("rustc: command not found", "rustc_not_found"),
            (
                "error: target may not be installed",
                "cargo_target_missing",
            ),
            (
                "pkg-config command could not be found",
                "cargo_pkg_config_missing",
            ),
            (
                "Could not find directory of OpenSSL installation and this `-sys` crate cannot proceed",
                "cargo_openssl_missing",
            ),
            ("go: command not found", "go_not_found"),
            ("go.sum: checksum mismatch", "go_checksum_mismatch"),
            (
                "These packages do not match the hashes from the requirements file",
                "pip_hash_mismatch",
            ),
            (
                "ERROR: Could not find a version that satisfies the requirement foo\nERROR: No matching distribution found for foo",
                "pip_no_matching_distribution",
            ),
            (
                "Ignored the following versions that require a different python version: 3.0 Requires-Python >=3.11",
                "pip_requires_python_mismatch",
            ),
            (
                "ProxyError: 407 Proxy Authentication Required",
                "pip_proxy_auth_required",
            ),
            (
                "HTTPSConnectionPool(host='pypi.org', port=443): Max retries exceeded with url: /simple",
                "pip_index_connection_failed",
            ),
            (
                "venv/bin/activate: No such file or directory",
                "venv_activation_missing",
            ),
            (
                "The engine \"node\" is incompatible with this module. Expected version \">=20\"",
                "node_engine_incompatible",
            ),
            (
                "npm ci can only install with an existing package-lock.json",
                "npm_ci_lock_missing",
            ),
            (
                "npm ERR! integrity checksum failed when using sha512",
                "npm_integrity_checksum_failed",
            ),
            (
                "npm ERR! code E401\nUnable to authenticate, need: Basic realm=\"npm\"",
                "npm_auth_e401",
            ),
            (
                "npm ERR! TAR_BAD_ARCHIVE: Unrecognized archive format",
                "npm_tar_unpack_error",
            ),
            ("yarn: command not found", "yarn_not_found"),
            ("pnpm: command not found", "pnpm_not_found"),
            (
                " ! [rejected] main -> main (non-fast-forward)",
                "git_non_fast_forward",
            ),
            (
                "Automatic merge failed; fix conflicts and then commit the result.",
                "git_merge_conflict",
            ),
            (
                "fatal: You are not currently on a branch.",
                "git_detached_head",
            ),
            ("Host key verification failed.", "git_host_key_verification_failed"),
            (
                "git-lfs filter-process: git-lfs: command not found",
                "git_lfs_not_installed",
            ),
            (
                "fatal: shallow update not allowed",
                "git_shallow_update_rejected",
            ),
            (
                "COPY failed: forbidden path outside the build context",
                "docker_build_context_forbidden",
            ),
            (
                "dockerfile parse error line 12: unknown instruction: runn",
                "dockerfile_parse_error",
            ),
            (
                "manifest for repo/demo:dev not found: manifest unknown",
                "docker_manifest_unknown",
            ),
            (
                "Error response from daemon: network not found",
                "docker_network_not_found",
            ),
            (
                "docker: 'buildx' is not a docker command.",
                "docker_buildx_not_found",
            ),
            (
                "pull access denied for private/app, repository does not exist or may require 'docker login'",
                "docker_login_required",
            ),
            ("gradle: command not found", "gradle_not_found"),
            (
                "maven: The JAVA_HOME environment variable is not defined correctly",
                "maven_java_home_not_defined",
            ),
            (
                "Could not resolve dependencies for project com.demo:app:jar:1.0.0",
                "maven_dependency_resolution_failed",
            ),
            (
                "Fatal error compiling: invalid target release: 21",
                "maven_invalid_target_release",
            ),
            ("rustup: command not found", "rustup_not_found"),
            (
                "error: toolchain 'stable-x86_64-unknown-linux-gnu' is not installed",
                "rust_toolchain_not_installed",
            ),
            (
                "error: the lock file needs to be updated but --frozen was passed to prevent this",
                "cargo_frozen_lockfile",
            ),
            (
                "error: failed to download from `https://index.crates.io/config.json`\nCaused by: operation timed out",
                "cargo_registry_timeout",
            ),
            (
                "failed to fetch into: /tmp/cargo-git\nauthentication required but no callback set",
                "cargo_git_fetch_auth",
            ),
            (
                "no required module provides package github.com/example/foo",
                "go_required_module_missing",
            ),
            (
                "System has not been booted with systemd as init system (PID 1). Can't operate.",
                "systemd_not_booted",
            ),
        ];

        for (log, expected) in fixtures {
            let hits = match_signatures(log);
            assert!(
                !hits.is_empty(),
                "fixture should hit at least one rule: {log}"
            );
            assert_eq!(
                hits[0].id, expected,
                "unexpected top hit for fixture: {log}"
            );
        }
    }

    #[test]
    fn should_detect_missing_prerequisites_from_snapshot() {
        let snapshot = EnvironmentSnapshot {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            shell: Some("bash".to_string()),
            elevated: false,
            path_preview: vec!["/usr/bin".to_string()],
            toolchains: vec![ToolVersion {
                name: "git".to_string(),
                version: "git version".to_string(),
            }],
            proxy: ProxySnapshot::default(),
            network: Vec::new(),
        };

        let hits = detect_environment_issues(&snapshot);
        let ids: HashSet<String> = hits.iter().map(|item| item.id.clone()).collect();
        assert!(ids.contains("dev_python_missing"));
        assert!(ids.contains("dev_c_compiler_missing"));
        assert!(ids.contains("dev_cpp_compiler_missing"));
    }

    #[test]
    fn should_detect_environment_mismatch() {
        let snapshot = EnvironmentSnapshot {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            shell: Some("bash".to_string()),
            elevated: false,
            path_preview: vec![
                "/usr/bin".to_string(),
                "/mnt/c/Windows/System32".to_string(),
            ],
            toolchains: Vec::new(),
            proxy: ProxySnapshot::default(),
            network: Vec::new(),
        };

        let hits = detect_environment_issues(&snapshot);
        assert!(hits
            .iter()
            .any(|item| item.id == "ide_environment_mismatch"));
    }

    #[test]
    fn should_have_reasonable_rule_coverage() {
        assert!(
            super::RULES.len() >= 80,
            "rule coverage dropped unexpectedly"
        );
    }

    #[test]
    fn should_smoke_match_each_rule_primary_pattern() {
        for rule in super::RULES {
            let first_pattern = rule.patterns.first().expect("rule must have patterns");
            let hits = match_signatures(first_pattern);
            assert!(
                hits.iter().any(|item| item.id == rule.id),
                "rule primary pattern should match itself: {}",
                rule.id
            );
        }
    }

    use std::collections::HashSet;
}
