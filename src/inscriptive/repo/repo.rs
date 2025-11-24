use super::repo_error::{RepoConstructionError, RepoInsertError};
use crate::{
    executive::program::{compiler::compiler::ProgramCompiler, program::Program},
    operative::Chain,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Guarded repo of contracts.
#[allow(non_camel_case_types)]
pub type PROGRAMS_REPO = Arc<Mutex<ProgramsRepo>>;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Directory for storing contract programs.
pub struct ProgramsRepo {
    programs: HashMap<CONTRACT_ID, Program>,
    programs_db: sled::Db,
}

impl ProgramsRepo {
    pub fn new(chain: Chain) -> Result<PROGRAMS_REPO, RepoConstructionError> {
        // Open the programs db.
        let programs_db = {
            let path = format!("{}/{}/{}", "storage", chain.to_string(), "repo");
            sled::open(path).map_err(|e| RepoConstructionError::DBOpenError(e))?
        };

        // Initialize the in-memory list of programs.
        let mut programs = HashMap::<CONTRACT_ID, Program>::new();

        // Collect the in-memory list of programs.
        for (index, lookup) in programs_db.iter().enumerate() {
            if let Ok((key, val)) = lookup {
                // Key is the 32-byte contract id.
                let contract_id: [u8; 32] = key
                    .as_ref()
                    .try_into()
                    .map_err(|_| RepoConstructionError::DBIterCollectInvalidKeyAtIndex(index))?;

                // Deserialize the program bytecode from value.
                let program_bytes: Vec<u8> = val.as_ref().to_vec();

                // Decompile the program from bytecode.
                let program: Program =
                    Program::decompile(&mut program_bytes.into_iter()).map_err(|e| {
                        RepoConstructionError::ProgramDecompileErrorAtKey(contract_id, e)
                    })?;

                // Insert into the in-memory programs list.
                programs.insert(contract_id, program);
            }
        }

        // Construct the repo.
        let repo = ProgramsRepo {
            programs,
            programs_db,
        };

        // Return the guarded repo.
        Ok(Arc::new(Mutex::new(repo)))
    }

    /// Inserts multiple programs into the repo.
    pub fn insert_multi(
        &mut self,
        programs: &HashMap<CONTRACT_ID, Program>,
    ) -> Result<(), RepoInsertError> {
        // Insert in-memory.
        for (contract_id, program) in programs {
            // Insert in-memory.
            if let Some(_) = self
                .programs
                .insert(contract_id.to_owned(), program.to_owned())
            {
                // Return error if the contract id already exists.
                return Err(RepoInsertError::ContractIdAlreadyExists(
                    contract_id.to_owned(),
                ));
            }
        }

        // Insert in-storage.
        for (contract_id, program) in programs {
            // Serialize the program.
            let program_bytes = program
                .compile()
                .map_err(|e| RepoInsertError::ProgramCompileError(contract_id.to_owned(), e))?;

            // Insert the program into the db.
            if let Some(_) = self
                .programs_db
                .insert(contract_id, program_bytes)
                .map_err(|e| RepoInsertError::DBInsertError(contract_id.to_owned(), e))?
            {
                // Return error if the contract id already exists.
                return Err(RepoInsertError::ContractIdAlreadyExists(
                    contract_id.to_owned(),
                ));
            }
        }

        // Return success.
        Ok(())
    }

    /// Returns the program by the contract id.
    pub fn program_by_contract_id(&self, contract_id: &CONTRACT_ID) -> Option<Program> {
        self.programs.get(contract_id).cloned()
    }

    /// Returns the number of methods in the program by the contract id.
    pub fn methods_len_by_contract_id(&self, contract_id: &CONTRACT_ID) -> Option<u8> {
        let methods_len = self
            .programs
            .get(contract_id)
            .map(|program| program.methods_len())?;

        // Return none if the methods length is greater than the maximum value of u8.
        if methods_len > u8::MAX as usize {
            return None;
        }

        // Return the methods length as u8.
        Some(methods_len as u8)
    }
}
