#[cfg(test)]
mod tests {
    use escrow_buy::instruction::*;

    // Instruction data unpack test
    #[test]
    fn instruction_unpack_test() {
        let mut inst_data = [
            0, 123, 0, 0, 0,
            0, 0, 0, 0 
        ];
        let mut result = EscrowInstruction::unpack(&inst_data).unwrap();
        assert_eq!(
            result, 
            EscrowInstruction::ListToken { amount: 123 }
        );

        inst_data = [
            1, 123, 0, 0, 0,
            0, 0, 0, 0 
        ];
        result = EscrowInstruction::unpack(&inst_data).unwrap();
        assert_eq!(
            result,
            EscrowInstruction::Exchange { amount: 123 }
        );
    
        inst_data = [
            2, 0, 0, 0, 0,
            0, 0, 0, 0 
        ];

        result = EscrowInstruction::unpack(&inst_data).unwrap();
        
        assert_eq!(
            result,
            EscrowInstruction::Cancel
        )
    }
}