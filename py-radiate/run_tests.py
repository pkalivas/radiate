#!/usr/bin/env python3
"""
Test runner script for Radiate Python package.

This script provides convenient ways to run different types of tests
with various configurations.
"""

import sys
import subprocess
import argparse
import os
from pathlib import Path


def run_command(cmd, description=""):
    """Run a command and handle errors."""
    print(f"\n{'='*60}")
    print(f"Running: {description}")
    print(f"Command: {' '.join(cmd)}")
    print(f"{'='*60}")
    
    try:
        subprocess.run(cmd, check=True, capture_output=False)
        print(f"\n✅ {description} completed successfully!")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n❌ {description} failed with exit code {e.returncode}")
        return False


def run_unit_tests(verbose=False, coverage=False):
    """Run unit tests."""
    cmd = ["python3", "-m", "pytest", "tests/unit/", "-m", "unit"]
    
    if verbose:
        cmd.append("-v")
    if coverage:
        cmd.extend(["--cov=radiate", "--cov-report=html", "--cov-report=term-missing"])
    
    return run_command(cmd, "Unit Tests")


def run_integration_tests(verbose=False, coverage=False):
    """Run integration tests."""
    cmd = ["python3", "-m", "pytest", "tests/integration/", "-m", "integration"]
    
    if verbose:
        cmd.append("-v")
    if coverage:
        cmd.extend(["--cov=radiate", "--cov-report=html", "--cov-report=term-missing"])
    
    return run_command(cmd, "Integration Tests")


def run_performance_tests(verbose=False, benchmark_only=False):
    """Run performance tests."""
    cmd = ["python3", "-m", "pytest", "tests/performance/", "-m", "performance"]
    
    if verbose:
        cmd.append("-v")
    if benchmark_only:
        cmd.append("--benchmark-only")
    
    return run_command(cmd, "Performance Tests")


def run_smoke_tests(verbose=False):
    """Run smoke tests."""
    cmd = ["python3", "-m", "pytest", "tests/", "-m", "smoke"]
    
    if verbose:
        cmd.append("-v")
    
    return run_command(cmd, "Smoke Tests")


def run_regression_tests(verbose=False):
    """Run regression tests."""
    cmd = ["python3", "-m", "pytest", "tests/", "-m", "regression"]
    
    if verbose:
        cmd.append("-v")
    
    return run_command(cmd, "Regression Tests")


def run_all_tests(verbose=False, coverage=False, exclude_slow=False):
    """Run all tests."""
    cmd = ["python3", "-m", "pytest", "tests/"]
    
    if verbose:
        cmd.append("-v")
    if coverage:
        cmd.extend(["--cov=radiate", "--cov-report=html", "--cov-report=term-missing"])
    if exclude_slow:
        cmd.extend(["-m", "not slow"])
    
    return run_command(cmd, "All Tests")


def run_linting():
    """Run code linting."""
    cmd = ["python3", "-m", "flake8", "radiate/", "tests/", "--max-line-length=100"]
    return run_command(cmd, "Code Linting")


def run_type_checking():
    """Run type checking."""
    cmd = ["python3", "-m", "mypy", "radiate/", "tests/"]
    return run_command(cmd, "Type Checking")


def run_security_check():
    """Run security checks."""
    cmd = ["python3", "-m", "bandit", "-r", "radiate/"]
    return run_command(cmd, "Security Check")


def install_test_dependencies():
    """Install test dependencies."""
    test_deps = [
        "pytest",
        "pytest-cov",
        "pytest-benchmark",
        "pytest-timeout",
        "pytest-xdist",
        "flake8",
        "mypy",
        "bandit",
        "psutil",
        "numpy", 
    ]
    
    cmd = ["pip3", "install"] + test_deps
    return run_command(cmd, "Installing Test Dependencies")


def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Radiate Test Runner")
    parser.add_argument("--unit", action="store_true", help="Run unit tests")
    parser.add_argument("--integration", action="store_true", help="Run integration tests")
    parser.add_argument("--performance", action="store_true", help="Run performance tests")
    parser.add_argument("--smoke", action="store_true", help="Run smoke tests")
    parser.add_argument("--regression", action="store_true", help="Run regression tests")
    parser.add_argument("--all", action="store_true", help="Run all tests")
    parser.add_argument("--lint", action="store_true", help="Run code linting")
    parser.add_argument("--type-check", action="store_true", help="Run type checking")
    parser.add_argument("--security", action="store_true", help="Run security checks")
    parser.add_argument("--install-deps", action="store_true", help="Install test dependencies")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--coverage", action="store_true", help="Generate coverage report")
    parser.add_argument("--benchmark-only", action="store_true", help="Only run benchmarks")
    parser.add_argument("--exclude-slow", action="store_true", help="Exclude slow tests")
    
    args = parser.parse_args()
    
    # Change to the script directory
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    print("🚀 Radiate Test Runner")
    print("=" * 60)
    
    success = True
    
    # Install dependencies if requested
    if args.install_deps:
        success &= install_test_dependencies()
    
    # Run specific test types
    if args.unit:
        success &= run_unit_tests(args.verbose, args.coverage)
    
    if args.integration:
        success &= run_integration_tests(args.verbose, args.coverage)
    
    if args.performance:
        success &= run_performance_tests(args.verbose, args.benchmark_only)
    
    if args.smoke:
        success &= run_smoke_tests(args.verbose)
    
    if args.regression:
        success &= run_regression_tests(args.verbose)
    
    if args.all:
        success &= run_all_tests(args.verbose, args.coverage, args.exclude_slow)
    
    # Run code quality checks
    if args.lint:
        success &= run_linting()
    
    if args.type_check:
        success &= run_type_checking()
    
    if args.security:
        success &= run_security_check()
    
    # If no specific tests were requested, run smoke tests by default
    if not any([args.unit, args.integration, args.performance, args.smoke, 
                args.regression, args.all, args.lint, args.type_check, 
                args.security, args.install_deps]):
        print("No specific tests requested, running smoke tests...")
        success &= run_smoke_tests(args.verbose)
    
    # Print summary
    print(f"\n{'='*60}")
    if success:
        print("🎉 All tests completed successfully!")
        sys.exit(0)
    else:
        print("❌ Some tests failed!")
        sys.exit(1)


if __name__ == "__main__":
    main() 