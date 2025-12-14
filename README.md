# cdisc-transpiler

```bash
. .venv/bin/activate
```

# Generate both XPT and Dataset-XML with Define-XML (default)

```bash
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/ -vv
```

```bash
cdisc-transpiler study mockdata/DEMO_CF1234_NL_20250120_104838/ -vv
```

https://library.cdisc.org/browser/#/mdr/sdtmig/3-4/classes/Interventions/datasets/AG

## Type checks (Pylance via pyright CLI)

```bash
pip install -e .[dev]
pyright
```
