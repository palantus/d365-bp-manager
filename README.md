# d365-bp-manager
Best Practices manager for D365

Create a `config.toml` file with something like:

```toml
modelpath = 'C:\\AOSService\\PackagesLocalDirectory'
models = [
	{name = 'MyModel1', alias = 'm1'},
	{name = 'MyModel2', alias = 'm2'},
]
```

Run with:
```
.\bp-manager.exe m2
```
