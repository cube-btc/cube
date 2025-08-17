# Transaction Outputs
Cube employs of 6 types of transaction outputs (TXOs):

| TXO Type               | Kind           |  Spending Condition                                        |
|:-----------------------|:---------------|:-----------------------------------------------------------|
| Lift 🛗                | Bare           | `(Self + Engine) or (Self after 3 months)`                 | 
| ZKTLC 💵               | Virtual        | `(Self + Engine) or (Self after 3 months)`                 |
| Projector 🎥           | Bare           | `(msg.senders[] + Engine) or (Engine after 3 months)`      |
| Payload 📦             | Bare           | `(msg.senders[] after 1 week) or (Engine with hashlocks)`  |
| Self 👨‍💻                | Virtual        | `(Self)`                                                   |
| Engine 🏭              | Virtual        | `(Engine)`                                                 |
 
Three of the transaction output types are virtual, meaning they are literal, on-chain transaction outputs that consume block space, while the other three are virtual, meaning they are committed but not yet revealed transaction outputs that optimistically consume no block space.

Cube advances the rollup state by chaining `Pool Transactions` at regular intervals. Two output types—`Projector`, and `Payload` and optionally one or more `Lifts` are contained in the `Pool Transaction`.

                                                                              ⋰
                                                                            ⋰  ┌────────────────┐
                                                                          ⋰    │    ZKTLC #0    │
                 Prevouts                       Outs                    ⋰      └────────────────┘
           ┌───────────────────┐      ┌─────────────────────┐         ⋰                 ┊                   
        #0 │      Payload      │   #0 │       Payload       │       ⋰          ┌────────────────┐
           └───────────────────┘      └─────────────────────┘     ⋰            │    ZKTLC #y    │
           ┌───────────────────┐      ┌─────────────────────┐   ⋰              └────────────────┘
        #1 │      Lift #0      │   #1 │      Projector      │ 🎥 ┈ ┈ ┈ ┈ ┈ ┈ ┈ ┈      
           └───────────────────┘      └─────────────────────┘         
                     ┊                ┌─────────────────────┐                          
           ┌───────────────────┐   #2 │       Lift #0       │ 
      #1+n │      Lift #n      │      └─────────────────────┘ 
           └───────────────────┘                  ┊             
                     ┊                ┌─────────────────────┐      
                                 #x+1 │       Lift #x       │       
                                      └─────────────────────┘         
                         Pool Transaction     

## Lift 🛗
`Lift` is a bare, on-chain transaction output type used for onboarding. When a `Lift` output is funded, it can be swapped out for a 1:1 `ZKTLC` in a process known as lifting. In short, `Lift` lifts itself up to a `ZKTLC`.

`Lift` carries two  spending conditions:
`(Self + Engine) or (Self after 3 months)`

-  **Lift Path:** `Self` and `Engine` sign from the collaborative path `(Self + Engine)` to swap the `Lift` output in exchange for a 1:1 `ZKTLC`.
    
-   **Exit Path:** In case the `Engine` is non-collaborative and does not sign from the collaborative path, `Self` can trigger the exit path `(Self after 3 months)` to reclaim their funds.

### External Funding
`Lift` is an on-chain P2TR address, so it can be funded by a third-party wallet, such as an exchange, a payroll service, or an individual. When a `Lift` output is funded by an external source, it must receive at least two on-chain confirmations to be considered valid.
                                                            
                                Prevouts                       Outs    
                         ┌────────────────────┐       ┌────────────────────┐  
                     #0  │     Third Party    │   #0  │         ...        │     
                         └────────────────────┘       └────────────────────┘
                                    ┊                            ┊
                         ┌────────────────────┐       ┌────────────────────┐ 
                     #x  │     Third Party    │   #y  │        Lift        │--->Pool Transaction
                         └────────────────────┘       └────────────────────┘                    
                                                                 ┊
                                                      ┌────────────────────┐
                                                  #z  │         ...        │
                                                      └────────────────────┘
      
                               A Third Party Payout Transaction 

### Internal Funding
`Lift` is can also be funded internally, within a pool transaction. When a `Lift` output is funded internally it can be spent in another pool transaction immediately.
                                                            
                                Prevouts                       Outs    

                                   ┊                            ┊ 
                                                      ┌────────────────────┐ 
                                                  #3  │       Lift #0      │--->Pool Transaction
                                                      └────────────────────┘                    
                                                                 ┊
                                                      ┌────────────────────┐
                                                #x+3  │       Lift #x      │--->Pool Transaction
                                                      └────────────────────┘
      
                                       Pool Transaction 

## ZKTLC 💵
`ZKTLC` is a virtual, off-chain transaction output that holds the `Self` funds. `ZKTLC` are projected by the `Projector` and can be unilaterally redeemed on-chain. A `ZKTLC` expires three months after its creation, or, in other words, three months after its projector `Projector` hits on-chain. 

Once a `VTXO` expires, it can no longer be redeemed or claimed on-chain; therefore, `Self` must either spend them entirely or refresh the `VTXOs` into new ones on a monthly basis. It is the client software's burden to abstract the refresh UX away for `Self`. At the protocol level, however, refreshes are interpreted differently from regular transfers, and the `Engine` is not allowed to charge liquidity fees when `VTXOs` are refreshed.

## Projector 🎥
`Projector` is a bare, on-chain transaction output type contained in each pool transaction. `Projector` projects `ZKTLCs` into a covenant template.
                                                      
                                           ⋰ ┌──────────────────┐
                                         ⋰   │     ZKTLC #0     │
                                       ⋰     └──────────────────┘
                                     ⋰       ┌──────────────────┐
                                   ⋰         │     ZKTLC #1     │
        ┌──────────────────┐     ⋰           └──────────────────┘
        │    Projector     │ 🎥 ⋮                        
        └──────────────────┘     ⋱                    ┊
                                   ⋱                
                                     ⋱       ┌──────────────────┐
                                       ⋱     │     ZKTLC #n     │
                                         ⋱   └──────────────────┘
                                           ⋱

`Projector` carries two spending conditions:
`(msg.senders[] + Engine) or (Engine after 3 months)`

-   **Reveal Path:** The aggregated [MuSig2](https://github.com/bitcoin/bips/blob/master/bip-0327.mediawiki) key of msg.senders[] and `Engine` pre-sign from the reveal path `(msg.senders[] + Engine)` to constrain `VTXOs` in a pseudo-covenant manner.
    
-  **Sweep Path:** `Projector` expires in three months, at which point all `ZKTLCs` contained within the projector also expire. Upon expiry, the `Engine` triggers the sweep path `(Engine after 3 months)` to reclaim all expired `ZKTLCs` directly from the projector root, in a footprint-minimal way, without claiming `ZKTLCs` one by one.          

## Payload 📦
`Payload` is a bare, on-chain transaction output type contained in each pool transaction.  `Payload` stores entries, projector signatures, s commitments, and the fresh engine key of the session.

## Self 👨‍💻
`Self` is virtually contained in `ZKTLCs`.

## Engine 🏭
`Engine` is virtually contained in `ZKTLCs`.