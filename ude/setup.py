"""
Universal Discovery Engine - Setup

Install: pip install -e .
Run: ude
"""

from setuptools import setup, find_packages

setup(
    name="universal-discovery-engine",
    version="0.2.0",
    description="Autonomous Mathematical Discovery System",
    author="UDE Team",
    packages=find_packages(),
    install_requires=[
        "numpy>=1.21.0",
    ],
    extras_require={
        "lean": ["lean==4.0.0"],
        "z3": ["z3-solver>=4.8.0"],
        "ml": ["torch>=2.0.0"],
        "all": ["lean==4.0.0", "z3-solver>=4.8.0", "torch>=2.0.0"],
    },
    entry_points={
        "console_scripts": [
            "ude=main:main",
        ],
    },
    python_requires=">=3.9",
)
