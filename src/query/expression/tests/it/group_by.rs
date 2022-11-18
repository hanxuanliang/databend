// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use common_exception::Result;
use common_expression::types::DataType;
use common_expression::types::NumberDataType;
use common_expression::*;

use crate::common::new_chunk;

#[test]
fn test_group_by_hash() -> Result<()> {
    let schema = DataSchemaRefExt::create(vec![
        DataField::new("a", SchemaDataType::Number(NumberDataType::Int8)),
        DataField::new("b", SchemaDataType::Number(NumberDataType::Int8)),
        DataField::new("c", SchemaDataType::Number(NumberDataType::Int8)),
        DataField::new("x", SchemaDataType::String),
    ]);

    let chunk = new_chunk(&vec![
        (
            DataType::Number(NumberDataType::Int8),
            Column::from_data(vec![1i8, 1, 2, 1, 2, 3]),
        ),
        (
            DataType::Number(NumberDataType::Int8),
            Column::from_data(vec![1i8, 1, 2, 1, 2, 3]),
        ),
        (
            DataType::Number(NumberDataType::Int8),
            Column::from_data(vec![1i8, 1, 2, 1, 2, 3]),
        ),
        (
            DataType::String,
            Column::from_data(vec!["x1", "x1", "x2", "x1", "x2", "x3"]),
        ),
    ]);

    let method = Chunk::choose_hash_method(&chunk, &[0, 3])?;
    assert_eq!(method.name(), HashMethodSerializer::default().name(),);

    let method = Chunk::choose_hash_method(&chunk, &[0, 1, 2])?;

    assert_eq!(method.name(), HashMethodKeysU32::default().name());

    let hash = HashMethodKeysU32::default();
    let columns = vec!["a", "b", "c"];

    let mut group_columns = Vec::with_capacity(columns.len());
    {
        for col in columns {
            let index = schema.index_of(col).unwrap();
            group_columns.push(chunk.column(index));
        }
    }

    todo!("expression");
    // let state = hash.build_keys_state(group_columns.as_slice(), chunk.num_rows())?;
    // let keys_iter = hash.build_keys_iter(&state)?;
    // let keys: Vec<u32> = keys_iter.copied().collect();
    // assert_eq!(keys, vec![
    //     0x10101, 0x10101, 0x20202, 0x10101, 0x20202, 0x30303
    // ]);
    Ok(())
}
