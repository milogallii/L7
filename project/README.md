# PROJECT STRUCTURE

# Cargo Workspaces

| Workspace        | Role |
| --------         | ------- |
| Simulation       | Runs the afxdp program that emulates a ship whose components have veths |
| Ship             | Ship structure emulation with higher level functions ( monitoring, firewall, ex...) |
| ShipComponent    | Ship components where for each one an afxdp socket is created whith every memory structure it needs   |

# Additional files

| File     | Utility |
| -------- | ------- |
| Makefile | Sets up the Linux namespaces for ship's components' veths |


