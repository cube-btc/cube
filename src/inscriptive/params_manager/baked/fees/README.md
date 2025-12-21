# Fees
Cube employs 6 distinct fee kinds:

| Fee Kind             | Description                                                                                    |
|:---------------------|:-----------------------------------------------------------------------------------------------|
| Base fees            | Base fee charged (1) per `Entry` and (2) per spent `Lift` txo.                                 |
| DA fees              | Fees paid for occupying Bitcoin data availability space in various `Entries`.                  |
| Execution fees       | Execution cost of various opcodes on running nodes.                                            |
| Liquidity fees       | Liquidity PPM fees involving (1) literal or (2) shadowed coin movements.                       |
| Registry fees        | Resource allocation cost for (1) onboarding a new `Account` or (2) deploying a new `Contract`. |
| Vitality fees        | Taxation of infrequent `Contract` calls for sustainable node resource allocation.              |
