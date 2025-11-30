# Python bindings to Rust crates in the rest of the repo

# Installation.

The following builds and installs the wheel that the poetry installed package uses to run said tests  
```poetry run maturin develop```  

The following installs the package so you can run tests with poetry run pytest. Only needed for tests really.  
```poetry install```     
  

Set working directory to minimal-fidl/minimal-fidl-python. Then run  
```poetry run pytest```  
to run the python tests in tests of this dir.