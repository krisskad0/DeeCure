# Risks and how to handle them

[Docs](../README.md) › [Product](README.md)

| Risk | What could happen | How to handle it |
| --- | --- | --- |
| A fix breaks a system | Users lose trust fast | Fixed action list, VM-tested fixes, undo points, clear "no undo" warnings |
| Wrong explanations | Beginners act on bad advice | Curated KB first; AI answers labeled; always show the source line |
| Too many alerts | Users ignore or uninstall | Only critical alerts push; everything else goes in the weekly summary |
| Hardware differences | Missing or odd sensor data | Show "not reported" honestly; collect fixtures from many laptops |
| Root helper becomes a security hole | Attackers use it | Small code, strict inputs, polkit, audit log, two-person review |
| Distro changes (log files, tools renamed) | Insights silently break | Adapters plus fixture tests per distro release |
