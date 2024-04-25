# d365-bp-manager
Best Practices manager for D365

## Screenshots

Model view:

![image](https://github.com/palantus/d365-bp-manager/assets/819342/07d3b36f-8a21-497f-9b83-c0c7d13bf270)

Best practices missing justifications:

![image](https://github.com/palantus/d365-bp-manager/assets/819342/a9a0831b-c984-4160-b87e-53940be77c48)

## Features

* Select model
* View best practices from last best practice check
* Type justification
* Write new suppression xml file

## Setup

Create a `config.toml` file with something like:

```toml
modelpath = 'C:\\AOSService\\PackagesLocalDirectory'
models = [
	'MyModelNO', 
	'MyModelCore'
]
```

Run with:
```
.\bp-manager.exe
```
