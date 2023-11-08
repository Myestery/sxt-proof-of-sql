use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedColumn,
    OwnedTable, SchemaAccessor, TableRef, TestAccessor,
};
use crate::base::scalar::{compute_commitment_for_testing, ArkScalar};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use indexmap::IndexMap;
use proofs_sql::Identifier;

#[derive(Default)]
/// A test accessor that uses OwnedTable as the underlying table type.
/// Note: this is not optimized for performance, so should not be used for benchmarks.
pub struct OwnedTableTestAccessor {
    tables: IndexMap<TableRef, (OwnedTable, usize)>,
    alloc: Bump,
}

impl Clone for OwnedTableTestAccessor {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
            ..Default::default()
        }
    }
}

impl TestAccessor for OwnedTableTestAccessor {
    type Table = OwnedTable;

    fn new_empty() -> Self {
        Default::default()
    }

    fn add_table(&mut self, table_ref: TableRef, data: OwnedTable, table_offset: usize) {
        self.tables.insert(table_ref, (data, table_offset));
    }

    fn get_column_names(&self, table_ref: TableRef) -> Vec<&str> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .column_names()
            .map(|id| id.as_str())
            .collect()
    }

    fn update_offset(&mut self, table_ref: TableRef, new_offset: usize) {
        self.tables.get_mut(&table_ref).unwrap().1 = new_offset;
    }
}
impl DataAccessor for OwnedTableTestAccessor {
    fn get_column(&self, column: ColumnRef) -> Column {
        match self
            .tables
            .get(&column.table_ref())
            .unwrap()
            .0
            .inner_table()
            .get(&column.column_id())
            .unwrap()
        {
            OwnedColumn::BigInt(col) => Column::BigInt(col),
            OwnedColumn::VarChar(col) => {
                let col: &mut [&str] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| s.as_str()));
                let scals: &mut [ArkScalar] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| s.into()));
                Column::VarChar((col, scals))
            }
            OwnedColumn::Int128(col) => Column::Int128(col),
        }
    }
}
impl CommitmentAccessor for OwnedTableTestAccessor {
    fn get_commitment(&self, column: ColumnRef) -> RistrettoPoint {
        let (table, offset) = self.tables.get(&column.table_ref()).unwrap();
        match table.inner_table().get(&column.column_id()).unwrap() {
            OwnedColumn::BigInt(vals) => compute_commitment_for_testing(vals, *offset),
            OwnedColumn::VarChar(vals) => compute_commitment_for_testing(vals, *offset),
            OwnedColumn::Int128(vals) => compute_commitment_for_testing(vals, *offset),
        }
    }
}
impl MetadataAccessor for OwnedTableTestAccessor {
    fn get_length(&self, table_ref: TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().0.num_rows()
    }

    fn get_offset(&self, table_ref: TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().1
    }
}
impl SchemaAccessor for OwnedTableTestAccessor {
    fn lookup_column(&self, table_ref: TableRef, column_id: Identifier) -> Option<ColumnType> {
        Some(
            self.tables
                .get(&table_ref)?
                .0
                .inner_table()
                .get(&column_id)?
                .column_type(),
        )
    }

    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .inner_table()
            .iter()
            .map(|(&id, col)| (id, col.column_type()))
            .collect()
    }
}