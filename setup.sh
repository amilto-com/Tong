#!/bin/bash
# TONG Programming Language Setup Script

echo "🚀 Setting up TONG - The Ultimate Programming Language"
echo "=================================================="

# Make main script executable
chmod +x tong.py

# Create symlink for global access (optional)
if [ "$1" = "--global" ]; then
    echo "Creating global symlink..."
    sudo ln -sf "$(pwd)/tong.py" /usr/local/bin/tong
    echo "✅ TONG is now available globally as 'tong'"
else
    echo "Run './tong.py' to start TONG"
    echo "Or run '$0 --global' to install globally"
fi

echo ""
echo "🎯 Quick Start:"
echo "  ./tong.py                    # Start REPL"
echo "  ./tong.py examples/hello.tong # Run example"
echo "  ./tong.py --help             # Show help"
echo ""
echo "📚 Examples available in examples/ directory"
echo "📖 See README.md for full documentation"
echo ""
echo "✨ TONG is ready for heterogeneous computing!"