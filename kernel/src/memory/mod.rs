
mod heap_allocator;

pub fn init(){
    heap_allocator::init_heap();
}

pub fn test(){
    heap_allocator::heap_test();   
}
