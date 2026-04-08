# Entries
`Entry` acts as a container for specific actions, such as calling a smart contract or moving coins.

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

## Entry Airly Payload Encoding (APE) Tree
                                                    
     ┌────────────────────────┐                               ┌────────────────────────┐     
     │ Common Branch          │                               │ Uncommon Branch        │
     │ b:0                    │                               │ b:1                    │
     └────────────────────────┘                               └────────────────────────┘        
            ┌────┘└────┐                        ┌─────────────────────────┘└─────────────────────────┐
     ┌───────────┐┌───────────┐    ┌────────────────────────┐                            ┌────────────────────────┐
     │ Move      ││ Call      │    │ Liquidity Branch       │                            │ Outer Branch           │  
     │ b:0       ││ b:1       │    │ b:0                    │                            │ b:1                    │
     └───────────┘└───────────┘    └────────────────────────┘                            └────────────────────────┘
                                          ┌────┘└────┐                         ┌──────────────────────┘└──────────────────────┐
                                   ┌───────────┐┌───────────┐     ┌────────────────────────┐                     ┌────────────────────────┐
                                   │ Add       ││ Sub       │     │ Gateway Branch         │                     │ Outer Right Branch     │  
                                   │ b:0       ││ b:1       │     │ b:0                    │                     │ b:1                    │
                                   └───────────┘└───────────┘     └────────────────────────┘                     └────────────────────────┘
                                                                         ┌────┘└────┐                      ┌─────────────────┘└─────────────────┐
                                                                  ┌───────────┐┌───────────┐  ┌────────────────────────┐              ┌────────────────────────┐
                                                                  │ Liftup    ││ Swapout   │  │ Outer Lowermost Branch │              │ Reserved Branch        │
                                                                  │ b:0       ││ b:1       │  │ b:0                    │              │ b:1                    │
                                                                  └───────────┘└───────────┘  └────────────────────────┘              └────────────────────────┘
                                                                                                     ┌────┘└────┐                            ┌────┘└────┐            
                                                                                              ┌───────────┐┌───────────┐              ┌───────────┐┌───────────┐
                                                                                              │ Deploy    ││ Config    │              │ Nop       ││ Fail      │
                                                                                              │ b:0       ││ b:1       │              │ b:0       ││ b:1       │
                                                                                              └───────────┘└───────────┘              └───────────┘└───────────┘


