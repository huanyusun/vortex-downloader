// Utilities for handling batch operations with partial success/failure

export interface BatchOperationResult<T> {
  successful: T[];
  failed: Array<{
    item: T;
    error: string;
  }>;
  totalCount: number;
  successCount: number;
  failureCount: number;
}

export interface BatchOperationOptions {
  continueOnError?: boolean;
  maxConcurrent?: number;
}

/**
 * Execute a batch operation with error handling
 */
export async function executeBatchOperation<TInput, TOutput>(
  items: TInput[],
  operation: (item: TInput) => Promise<TOutput>,
  options: BatchOperationOptions = {}
): Promise<BatchOperationResult<TInput>> {
  const { continueOnError = true, maxConcurrent = 5 } = options;
  
  const successful: TOutput[] = [];
  const failed: Array<{ item: TInput; error: string }> = [];
  let shouldStop = false;
  
  // Process items in batches to avoid overwhelming the system
  for (let i = 0; i < items.length && !shouldStop; i += maxConcurrent) {
    const batch = items.slice(i, i + maxConcurrent);
    
    const results = await Promise.allSettled(
      batch.map(item => operation(item))
    );
    
    results.forEach((result, index) => {
      const item = batch[index];
      
      if (result.status === 'fulfilled') {
        successful.push(result.value);
      } else {
        const errorMessage = result.reason?.message || String(result.reason) || 'Unknown error';
        failed.push({ item, error: errorMessage });
        
        if (!continueOnError) {
          shouldStop = true;
        }
      }
    });
  }
  
  return {
    successful: successful as any,
    failed,
    totalCount: items.length,
    successCount: successful.length,
    failureCount: failed.length,
  };
}

/**
 * Format a batch operation result into a user-friendly message
 */
export function formatBatchResultMessage(result: BatchOperationResult<any>): {
  title: string;
  message: string;
  type: 'success' | 'warning' | 'error';
} {
  const { successCount, failureCount, totalCount } = result;
  
  if (failureCount === 0) {
    return {
      title: 'Success',
      message: `Successfully added ${successCount} video${successCount !== 1 ? 's' : ''} to queue`,
      type: 'success',
    };
  }
  
  if (successCount === 0) {
    return {
      title: 'Failed',
      message: `Failed to add ${failureCount} video${failureCount !== 1 ? 's' : ''} to queue`,
      type: 'error',
    };
  }
  
  return {
    title: 'Partially Completed',
    message: `Added ${successCount} of ${totalCount} videos. ${failureCount} failed.`,
    type: 'warning',
  };
}

/**
 * Get detailed error messages for failed items
 */
export function getFailedItemsDetails<T>(
  failed: Array<{ item: T; error: string }>,
  getItemName: (item: T) => string
): string[] {
  return failed.map(({ item, error }) => {
    const name = getItemName(item);
    return `${name}: ${error}`;
  });
}

/**
 * Categorize errors by type for better reporting
 */
export interface ErrorCategory {
  type: string;
  count: number;
  message: string;
  items: string[];
}

export function categorizeErrors<T>(
  failed: Array<{ item: T; error: string }>,
  getItemName: (item: T) => string
): ErrorCategory[] {
  const categories = new Map<string, ErrorCategory>();
  
  failed.forEach(({ item, error }) => {
    const itemName = getItemName(item);
    
    // Categorize by error type
    let errorType = 'Unknown Error';
    if (error.toLowerCase().includes('network')) {
      errorType = 'Network Error';
    } else if (error.toLowerCase().includes('unavailable') || error.toLowerCase().includes('not found')) {
      errorType = 'Video Unavailable';
    } else if (error.toLowerCase().includes('permission') || error.toLowerCase().includes('access')) {
      errorType = 'Access Denied';
    } else if (error.toLowerCase().includes('invalid')) {
      errorType = 'Invalid URL';
    } else if (error.toLowerCase().includes('timeout')) {
      errorType = 'Timeout';
    }
    
    if (!categories.has(errorType)) {
      categories.set(errorType, {
        type: errorType,
        count: 0,
        message: error,
        items: [],
      });
    }
    
    const category = categories.get(errorType)!;
    category.count++;
    category.items.push(itemName);
  });
  
  return Array.from(categories.values()).sort((a, b) => b.count - a.count);
}
