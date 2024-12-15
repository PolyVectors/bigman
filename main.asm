        INP
        STA A
        INP
        STA B
        LDA A
        SUB B
        BRP isPositive
        LDA A
        OUT
        HLT
isPositive LDA B
        OUT
        HLT
A       DAT
B       DAT
