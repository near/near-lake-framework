/// Type alias represents the block height
pub type BlockHeight = u64;

/// Configuration struct for NEAR Lake Framework
/// NB! Consider using [`LakeBuilder`]
/// Building the `Lake` example:
/// ```
/// use near_lake_framework::LakeBuilder;
///
/// # fn main() {
///    let lake = LakeBuilder::default()
///        .testnet()
///        .start_block_height(82422587)
///        .build()
///        .expect("Failed to build Lake");
/// # }
/// ```
#[derive(Default, Builder, Debug)]
#[builder(pattern = "owned")]
pub struct Lake {
    /// AWS S3 Bucket name
    #[builder(setter(into))]
    pub(crate) s3_bucket_name: String,
    /// AWS S3 Region name
    #[builder(setter(into))]
    pub(crate) s3_region_name: String,
    /// Defines the block height to start indexing from
    pub(crate) start_block_height: u64,
    /// Custom aws_sdk_s3::config::Config
    /// ## Use-case: custom endpoint
    /// You might want to stream data from the custom S3-compatible source () . In order to do that you'd need to pass `aws_sdk_s3::config::Config` configured
    /// ```
    /// use near_lake_framework::LakeBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let aws_config = aws_config::from_env().load().await;
    ///     let mut s3_conf = aws_sdk_s3::config::Builder::from(&aws_config)
    ///         .endpoint_url("http://0.0.0.0:9000")
    ///         .build();
    ///
    ///     let lake = LakeBuilder::default()
    ///         .s3_config(s3_conf)
    ///         .s3_bucket_name("near-lake-data-custom")
    ///         .s3_region_name("eu-central-1")
    ///         .start_block_height(1)
    ///         .build()
    ///         .expect("Failed to build Lake");
    /// # }
    /// ```
    #[builder(setter(strip_option), default)]
    pub(crate) s3_config: Option<aws_sdk_s3::config::Config>,
    /// Defines how many *block heights* Lake Framework will try to preload into memory to avoid S3 `List` requests.
    /// Default: 100
    ///
    /// *Note*: This value is not the number of blocks to preload, but the number of block heights.
    /// Also, this value doesn't affect your indexer much if it follows the tip of the network.
    /// This parameter is useful for historical indexing.
    #[builder(default = "100")]
    pub(crate) blocks_preload_pool_size: usize,
    /// Number of concurrent blocks to process. Default: 1
    /// **WARNING**: Increase this value only if your block handling logic doesn't have to rely on previous blocks and can be processed in parallel
    #[builder(default = "1")]
    pub(crate) concurrency: usize,
}

impl LakeBuilder {
    /// Shortcut to set up [LakeBuilder::s3_bucket_name] for mainnet
    /// ```
    /// use near_lake_framework::LakeBuilder;
    ///
    /// # fn main() {
    ///    let lake = LakeBuilder::default()
    ///        .mainnet()
    ///        .start_block_height(65231161)
    ///        .build()
    ///        .expect("Failed to build Lake");
    /// # }
    /// ```
    pub fn mainnet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-mainnet".to_string());
        self.s3_region_name = Some("eu-central-1".to_string());
        self
    }

    /// Shortcut to set up [LakeBuilder::s3_bucket_name] for testnet
    /// ```
    /// use near_lake_framework::LakeBuilder;
    ///
    /// # fn main() {
    ///    let lake = LakeBuilder::default()
    ///        .testnet()
    ///        .start_block_height(82422587)
    ///        .build()
    ///        .expect("Failed to build Lake");
    /// # }
    /// ```
    pub fn testnet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-testnet".to_string());
        self.s3_region_name = Some("eu-central-1".to_string());
        self
    }

    /// Shortcut to set up [LakeBuilder::s3_bucket_name] for betanet
    /// ```
    /// use near_lake_framework::LakeBuilder;
    ///
    /// # fn main() {
    ///    let lake = LakeBuilder::default()
    ///        .betanet()
    ///        .start_block_height(82422587)
    ///        .build()
    ///        .expect("Failed to build Lake");
    /// # }
    /// ```
    pub fn betanet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-betanet".to_string());
        self.s3_region_name = Some("us-east-1".to_string());
        self
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum LakeError {
    #[error("Failed to parse structure from JSON: {error_message}")]
    ParseError {
        #[from]
        error_message: serde_json::Error,
    },
    #[error("AWS S3 error: {error}")]
    AwsGetObjectError {
        #[from]
        error: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    },
    #[error("AWS S3 error: {error}")]
    AwsLisObjectsV2Error {
        #[from]
        error:
            aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>,
    },
    #[error("Failed to convert integer: {error}")]
    IntConversionError {
        #[from]
        error: std::num::TryFromIntError,
    },
    #[error("Join error: {error}")]
    JoinError {
        #[from]
        error: tokio::task::JoinError,
    },
    #[error("Failed to start runtime: {error}")]
    RuntimeStartError {
        #[from]
        error: std::io::Error,
    },
    #[error("Internal error: {error_message}")]
    InternalError { error_message: String },
}

/// ### The concept of Context for the Lake Framework
/// The main idea of the Lake Framework is to provide a simple way to index data from the NEAR blockchain.
/// The framework is designed to be as flexible as possible, so it doesn't provide any specific logic for indexing.
/// Instead, it provides a way to implement your own logic. One of the main concepts of the framework is the Context.
/// The Context is a struct that implements the [LakeContext] trait. It is used to pass data between the framework and your logic.
/// The Context is created once and then passed to the framework. The framework will call the [LakeContext::execute_before_run]
/// method before the indexing process starts and [LakeContext::execute_after_run] after the indexing process is finished.
/// The Context is useful for passing data between blocks. For example, you can use it to store the last block timestamp and use it in the next block.
///
/// Also the Context is necessary to pass the "global" data to the indexing process. For example, you can use it to pass the database connection pool.
///
/// ### Examples
///
/// #### Simple Context examples (explicit)
/// **WARNING**: This example demonsrates how Context works explicitly. In the real-world application you would do less boilerplate. See further examples.
/// In this example we will create a simple Context that prints the block height before the processing the block.
/// ```no_run
/// use near_lake_framework::LakeContextExt; // note Lake Framework exports this trait with a suffix Ext in the name
/// struct PrinterContext;
///
/// impl LakeContextExt for PrinterContext {
///    fn execute_before_run(&self, block: &near_lake_primitives::block::Block) {
///       println!("Processing block {}", block.header().height());
///   }
///   fn execute_after_run(&self) {}
/// }
/// ```
/// As you can see we will be printing `Processing block {block_height}` before processing the block. And we will do nothing after
///  the indexing process is finished.
///
/// The next example is showing how to provide some value to the indexing process.
/// ```no_run
/// use near_lake_framework::LakeContextExt; // note Lake Framework exports this trait with a suffix Ext in the name
/// use near_lake_framework::LakeBuilder;
/// # use diesel::Connection;
///
/// struct ApplicationDataContext {
///    pub db_pool: diesel::pg::PgConnection,
/// }
///
/// // We need our context to do nothing before and after the indexing process.
/// // The only purpose is to provide the database connection pool to the indexing process.
/// impl LakeContextExt for ApplicationDataContext {
///   fn execute_before_run(&self, block: &near_lake_primitives::block::Block) {}
///   fn execute_after_run(&self) {}
/// }
///
/// fn main() {
///     let db_pool = diesel::PgConnection::establish("postgres://localhost:5432")
///        .expect("Failed to connect to database");
///     let context = ApplicationDataContext { db_pool };
///
///     let result = LakeBuilder::default()
///       .testnet()
///       .start_block_height(82422587)
///       .build()
///       .unwrap()
///       .run_with_context(indexing_function, &context);
/// }
///
/// async fn indexing_function(
///    block: near_lake_primitives::block::Block,
///    context: &ApplicationDataContext,
/// ) -> Result<(), near_lake_framework::LakeError> {
///     // Now we can use the database connection pool
///     let db_pool = &context.db_pool;
///     ///...
///     Ok(())
/// }
/// ```
///
/// #### Simple Context example (real-world)
/// The last example from the previous section is a bit verbose. In the real-world application you would do less boilerplate.
/// The main purpose of that example was to show you what's happening under the hood. However, for your convenience, the Lake Framework
/// provides a trait [LakeContextExt] that implements the [LakeContext] trait for you. So you can use it to create a simple Context.
///
/// ```ignore
/// use near_lake_framework::LakeContext; // This is a derive macro
/// use near_lake_framework::LakeBuilder;
///
/// #[derive(LakeContext)]
/// /// struct ApplicationDataContext {
///    pub db_pool: diesel::pg::PgConnection,
/// }
///
/// // Here we got rid of the boilerplate code that we had in the previous example to impl the LakeContext trait.
///
/// fn main() {
///     let db_pool = diesel::pg::PgConnection::establish("postgres://postgres:password@localhost:5432/database")
///        .unwrap_or_else(|_| panic!("Error connecting to database"))
///
///     let context = ApplicationDataContext { db_pool };
///
///     let result = LakeBuilder::default()
///       .testnet()
///       .start_block_height(82422587)
///       .build()
///       .unwrap()
///       .run_with_context(indexing_function, &context);
/// }
///
/// async fn indexing_function(
///    block: near_lake_primitives::block::Block,
///    context: &ApplicationDataContext,
/// ) -> Result<(), near_lake_framework::LakeError> {
///     // Now we can use the database connection pool
///     let db_pool = &context.db_pool;
///     // ...
///    Ok(())
/// }
/// ```
///
/// It might look like not a big deal to get rid of the boilerplate code. However, it is very useful when you have a lot of Contexts or when you
/// use a ready-to-use Context from the community.
///
/// #### Advanced Context example
/// In this example we will extend a previous one with the `ParentTransactionCache` context Lake Framework team has created and shared with everybody.
///
/// ```ignore
/// use near_lake_framework::LakeContext; // This is a derive macro
/// use near_lake_parent_transaction_cache::{ParentTransactionCache, ParentTransactionCacheBuilder}; // This is a ready-to-use Context from the community that impls LakeContext trait
/// use near_lake_framework::LakeBuilder;
/// # use diesel::Connection;
///
/// #[derive(LakeContext)]
/// struct ApplicationDataContext {
///    pub db_pool: diesel::pg::PgConnection,
///   pub parent_transaction_cache: ParentTransactionCache,
/// }
///
/// fn main() {
///     let db_pool = diesel::PgConnection::establish("postgres://postgres:password@localhost:5432/database")
///        .unwrap_or_else(|_| panic!("Error connecting to database"));
///     let parent_transaction_cache = ParentTransactionCacheBuilder::default().build().unwrap();
///
///     let context = ApplicationDataContext { db_pool, parent_transaction_cache };
///
///     let result = LakeBuilder::default()
///       .testnet()
///       .start_block_height(82422587)
///       .build()
///       .unwrap()
///       .run_with_context(indexing_function, &context);
/// }
///
/// async fn indexing_function(
///    block: near_lake_primitives::block::Block,
///    context: &ApplicationDataContext,
/// ) -> Result<(), near_lake_framework::LakeError> {
///     // Now we can use the database connection pool
///     let db_pool = &context.db_pool;
///     dbg!(&context.parent_transaction_cache);
///     Ok(())
/// }
/// ```
/// As you can see we have extended our context with the `ParentTransactionCache` context. And we can use it in our indexing function.
/// The `ParentTransactionCache` defines the `execute_before_run` and `execute_after_run` methods. So when we call `run_with_context` method
/// the Lake Framework will call `execute_before_run` and `execute_after_run` methods for us.
/// And we didn't need to implement them in our `ApplicationDataContext` struct because `LakeContext` derive macro did it for us automatically.
pub trait LakeContextExt {
    /// This method will be called before the indexing process is started.
    fn execute_before_run(&self, block: &near_lake_primitives::block::Block);
    /// This method will be called after the indexing process is finished.
    fn execute_after_run(&self);
}
