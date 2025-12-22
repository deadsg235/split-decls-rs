generate_addr_of_methods ! { impl <> TimerShared { unsafe fn addr_of_pointers (self : NonNull < Self >) -> NonNull < linked_list :: Pointers < TimerShared >> { & self . pointers}
} }