# Entity
Cube supports 2 types of entities:

| Entity           | Description                                                                                      |
|:-----------------|:-------------------------------------------------------------------------------------------------|
| Account 👨‍💻       | Represents a distinct user within the system.                                                    |
| Contract 📑      | Represents a program within the system that can be called by an `Account` or another `Contract`. |

## Account 👨‍💻
An `Account` is a user-controlled entity that serves as the primary actor within the system. It can initiate calls to `Contract`s to execute program logic or move satoshis to other `Account`s.

## Contract 📑
A `Contract` is an executable program within the system. It can be called by `Account`s to perform specific actions. A `Contract` can also call other `Contract`s, enabling composable interactions and supporting complex functionality.
