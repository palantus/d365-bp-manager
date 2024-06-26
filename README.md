# Best Practices manager for D365

This is a graphical terminal app written in Rust, that enables you to get an overview of the output from the last BP-check and write justifications.

## Screenshots

Select model:

![image](https://github.com/palantus/d365-bp-manager/assets/819342/07d3b36f-8a21-497f-9b83-c0c7d13bf270)

Best practices:

![image](https://github.com/palantus/d365-bp-manager/assets/819342/a9a0831b-c984-4160-b87e-53940be77c48)

## Features

* Select model
* View best practices from last best practice check
* Write justification
* Generate new suppression xml file with your new justifications

## Installation

Download from Releases and extract zip file.

## Setup

Create a `config.toml` file with something like:

```toml
modelpath = 'C:\\AOSService\\PackagesLocalDirectory'
models = ['MyModelNO', 'MyModelCore']
```

Run with:
```
.\bp-manager.exe
```
