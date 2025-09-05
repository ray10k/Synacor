#define WORD int

//Initially: 
// r0 = 0x70BA
// r1 = 0x0611
// r2 = 0x3FE8

WORD mystery_function(r0, r1, r2) {
    //Registers 0 and 3 through 6 are saved.
    array_start = r0;//only point where r6 is set.
    print_obf_char = r1;//only point where r5 is set.
    loop_count = *r0; //treat r0 as an address, read memory. r4 does not get set again after.
    for(r1 = 0; r1 != 0; ++r1) {
        r3 = r1 + 1;
        if (r3 > loop_count){
            break;
        }
        r3 += array_start;
        r0 = *r3;
        print_obf_char(r0, r2);
    }
    //restore registers 0 and 3 through 6.
}

WORD print_obf_char(r0, r2) {
    //register 1 is saved.
    r1 = r2;
    r0 = decode_char(r0, r1);
    print_char(r0);
    //register 1 is restored.
    return;
}

WORD decode_char(r0, r1) {
    //registers 1 and 2 are saved.
    r2 = ~(r0 & r1);
    r0 = (r0 | r1) & r2;
    //restore registers 1 and 2.
    return r0;
}