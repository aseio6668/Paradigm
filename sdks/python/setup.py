from setuptools import setup, find_packages
import os

# Read README.md for long description
def read_readme():
    with open("README.md", "r", encoding="utf-8") as fh:
        return fh.read()

# Read requirements.txt
def read_requirements():
    with open("requirements.txt", "r", encoding="utf-8") as fh:
        return [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="paradigm-sdk",
    version="1.0.0",
    author="Paradigm Network",
    author_email="support@paradigm.network",
    description="Official Python SDK for Paradigm blockchain network",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/paradigm-network/paradigm-sdk-python",
    project_urls={
        "Bug Tracker": "https://github.com/paradigm-network/paradigm-sdk-python/issues",
        "Documentation": "https://docs.paradigm.network/sdk/python",
        "Homepage": "https://paradigm.network",
    },
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Internet :: WWW/HTTP",
        "Topic :: Office/Business :: Financial",
        "Topic :: System :: Distributed Computing",
    ],
    python_requires=">=3.8",
    install_requires=read_requirements(),
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0",
            "black>=23.0",
            "isort>=5.12",
            "flake8>=6.0",
            "mypy>=1.0",
            "sphinx>=6.0",
            "sphinx-rtd-theme>=1.2",
        ],
        "websocket": [
            "websockets>=11.0",
        ],
    },
    keywords=[
        "paradigm",
        "blockchain",
        "cryptocurrency",
        "web3",
        "sdk",
        "api",
        "defi",
        "smart-contracts",
    ],
    include_package_data=True,
    zip_safe=False,
)