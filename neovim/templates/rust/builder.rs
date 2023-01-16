pub struct Builder {
    data: Data,
}

// impl Default if possible, otherwise new()

impl Builder {
    pub fn data(self, data: Data) -> Self {
        Builder { data, ..self }
    }

    pub fn build(self) -> Result<(), Error> {
        Ok(())
    }
}
