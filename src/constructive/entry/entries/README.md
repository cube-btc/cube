### Entries
Cube employs 10 core `Entry` types:

| Entry            | Description                                                           |
|:-----------------|:----------------------------------------------------------------------|
| Move 💸          | Moves coins from an `Account` to another `Account`.                   |
| Call 📡          | Calls a `Contract`. This may internally involve moving coins.         |
| Add ➕           | Adds liquidity to `Engine`.                                           |
| Sub ➖           | Removes liquidity from `Engine`.                                      |
| Liftup ⬆️        | Lifts one or more `Lift` Bitcoin previous transaction outputs.        |
| Swapout 🚪       | Swaps `Account`'s coins 1:1 into a bare Bitcoin transaction output.   |
| Deploy 🏗        | Deploys a `Contract`.                                                 |
| Config ⚙️        | Configures or re-configures an `Account`.                             |
| Nop 📁           | Does nothing. Reserved for future upgrades.                           |
| Fail 📁          | Fails the `Entry`. Reserved for future upgrades.                      |

