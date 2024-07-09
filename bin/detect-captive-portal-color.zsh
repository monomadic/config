#!/usr/bin/env zsh

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print colored and formatted text
print_formatted() {
  local color=$1
  local text=$2
  echo -e "${color}${BOLD}${text}${NC}"
}

# Function to check if a command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Print script header
print_formatted $MAGENTA "\n=== Captive Portal Detector ==="
print_formatted $CYAN "Checking for required commands..."

# Check for required commands
for cmd in curl dig nc; do
  if ! command_exists $cmd; then
    print_formatted $RED "Error: $cmd is not installed. Please install it and try again."
    exit 1
  fi
done

print_formatted $GREEN "All required commands are available."

# Define variables
TEST_URL="http://connectivitycheck.gstatic.com/generate_204"
DNS_TEST_DOMAIN="www.google.com"
TIMEOUT=5

print_formatted $CYAN "\nInitiating captive portal detection..."

# 1. DNS resolution check
print_formatted $YELLOW "\n1. Performing DNS resolution check..."
if ! dig +short $DNS_TEST_DOMAIN >/dev/null 2>&1; then
  print_formatted $RED "   ✗ DNS resolution failed. Possible captive portal."
else
  print_formatted $GREEN "   ✓ DNS resolution successful."
fi

# 2 & 3. HTTP request and response analysis
print_formatted $YELLOW "\n2. Sending HTTP request and analyzing response..."
RESPONSE=$(curl -sS -m $TIMEOUT -o /dev/null -w "%{http_code} %{redirect_url}" $TEST_URL)
STATUS_CODE=$(echo $RESPONSE | cut -d' ' -f1)
REDIRECT_URL=$(echo $RESPONSE | cut -d' ' -f2-)

if [[ $STATUS_CODE != "204" ]]; then
  print_formatted $RED "   ✗ Unexpected status code: $STATUS_CODE. Possible captive portal."
  if [[ -n $REDIRECT_URL ]]; then
    print_formatted $MAGENTA "   → Captive portal URL: $REDIRECT_URL"
  fi
else
  print_formatted $GREEN "   ✓ HTTP check passed. No captive portal detected."
fi

# 4. Connectivity test
print_formatted $YELLOW "\n3. Performing TCP connection test..."
if ! nc -z -w $TIMEOUT www.google.com 80 >/dev/null 2>&1; then
  print_formatted $RED "   ✗ TCP connection test failed. Possible captive portal."
else
  print_formatted $GREEN "   ✓ TCP connection test passed."
fi

print_formatted $MAGENTA "\n=== Captive portal detection complete ==="

# Summary
print_formatted $CYAN "\nSummary:"
if [[ $STATUS_CODE == "204" ]] && nc -z -w $TIMEOUT www.google.com 80 >/dev/null 2>&1; then
  print_formatted $GREEN "No captive portal detected. You appear to have full internet access."
else
  print_formatted $YELLOW "A captive portal may be present. Please check your network connection."
  if [[ -n $REDIRECT_URL ]]; then
    print_formatted $MAGENTA "You may need to visit: $REDIRECT_URL"
  fi
fi

echo # Print a newline for better spacing
