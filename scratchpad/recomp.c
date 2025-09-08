#define WORD int
#define WORD_MAX 0x7fff

void main() {
    //the rest of the program happens here.
    //Starting analysis from address 0x1561
    save(r0,r1,r2);
    if (!r7) {return;} //continues normal execution; r7 not set.
    save(r0,r1,r2);
    r0 = 0x70ba;
    r1 = 0x0611;
    r2 = 0x3fe8; //note: was encoded as 0x3fe5 + 3
    display_message(r0,r1,r2); //subroutine at 0x05c8
    load(r0,r1,r2);
    //addresses 0x157c through 0x1580 are NOP
    r0 = 4;
    r1 = 1;
    r0 = func_17a1(r0,r1,r7);
    if (r0 == 6) {
        save(r0,r1,r2);
        r0 = 0x7163;
        r1 = 0x0611;
        r2 = 0x59EB; //note: was encoded as 0x1f73 + 0x3a78
        display_message(r0,r1,r2);
        load(r0,r1,r2); //note: this is pointless. Could have skipped the previous save.
        r0 = r7; //note: this is a point of interest!
        r1 = 0x6518;
        r2 = 0x7fff; //note: highest possible WORD value.
        save(r3);
        r3 = 0x7246;
        func_0747(r0,r1,r2,r3);
        load(r3);
        save(r0,r1,r2);
        r0 = 0x724a;
        r1 = 0x0611;
        r2 = 0x74F1; //encoded as 0x00ca + 0x7427
        display_message(r0,r1,r2);
        mem[0x0ac2] = 0x09d8;
        mem[0x0ac3] = 0;
        mem[0x0aac] = 0x7fff; //Note: address was calculated rather than stored as constant.
    }
    else {
        save(r0,r1,r2);
        r0 = 0x727b;
        r1 = 0x0611;
        r2 = 0x30f1; //encoded as 0x01aa + 0x2f47
        display_message(r0,r1,r2);
        load(r0,r1,r2);
    }
    load(r0,r1,r2);
    return
}

void func_0747(a,b,c,src_addr) {
    save(r3,r4,r5,r6);//Note: r1 and r2 are not saved. Constants?
    len = mem[0x1803]; //hack to make this work.
    for (i = i; i < len; i++){
        mem[0x1803 + i] = mem[src_addr+i]; //memory copy?
    }
    outer: {//address 0x076e
        r3 = 0;
        r4 = 0;
        inner: {//address 0x0774
            r5 = mem[0x1803];
            r5 = r4 % r5;
            r5 += 0x1803;
            r5 += 1;
            r6 = mem[r5];
            r6 = (r6 * 0x1481) % WORD_MAX;
            r6 = (r6 + 0x3039) % WORD_MAX;
            mem[r5] = r6;
            save(r0,r1);
            r1 = r6;
            r6 = xor(r0,r1);
            load(r0,r1);
            r1 = mem[r5];
            r6 = r6 % r5;
            r6 += 1;
            if (r5 <= 0x7b6) {
                r3 = 1;
            }
            r6 = (r6 + r1) % WORD_MAX;
            r6 = mem[r6];
            r4 += 1;
            r5 = r4 + 0x1807;
            mem[r5] = r6;
            r5 = mem[0x1807];
            if (r4 != r5) {
                goto inner;
            }
        }
        if (r3 == 0) {
            goto outer;
        }   
    }
    save(r0)
    r0 = 0x1807;
    func_0604(r0);
    load(r0,r3,r4,r5,r6);
    return;
}

WORD xor(a, b) {
    return a ^ b;
}

WORD func_17a1(a, b, c) { //Seems to do a bunch of recursive operations on a and b, with c being a constant.
    if (a != 0) {
        if (b != 0) {
            save(a);
            a = (a + 0x7fff) % WORD_MAX;
            b = func_17a1(a,b,c)
            load(a);
            a = (a + 0x7fff) % WORD_MAX;
            return func_17a1(a,b,c);
        }
        else {
            a = (a + 0x7fff) % WORD_MAX;
            b = c;
            a = func_17a1(a,b,c);
            return a;
        }
    }
    return b + 1; 
}