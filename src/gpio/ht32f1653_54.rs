use super::GpioRegExt;

gpio_trait!(gpioa: gpioa_dinr, gpioa_doutr, gpioa_srr, gpioa_rr);
gpio_trait!(gpiob: gpiob_dinr, gpiob_doutr, gpiob_srr, gpiob_rr);
gpio_trait!(gpioc: gpioc_dinr, gpioc_doutr, gpioc_srr, gpioc_rr);
gpio_trait!(gpiod: gpiod_dinr, gpiod_doutr, gpiod_srr, gpiod_rr);

gpio!(GPIOA, gpioa, parst, paen, gpioa_drvr, gpioa_dircr, gpioa_pur, gpioa_pdr, gpioa_iner, gpioa_odr, [
    PA0: (pa0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PA1: (pa1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PA2: (pa2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PA3: (pa3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PA4: (pa4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PA5: (pa5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PA6: (pa6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PA7: (pa7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    // BOOT0
    PA8: (pa8, 8, Input<PullUp>, AF0, dir8, pu8, pd8, inen8, od8),
    // BOOT1
    PA9: (pa9, 9, Input<PullUp>, AF0, dir9, pu9, pd9, inen9, od9),
    PA10: (pa10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PA11: (pa11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    // SWCLK
    PA12: (pa12, 12, Input<PullUp>, AF0, dir12, pu12, pd12, inen12, od12),
    // SWDIO
    PA13: (pa13, 13, Input<PullUp>, AF0, dir13, pu13, pd13, inen13, od13),
    PA14: (pa14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PA15: (pa15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOB, gpiob, pbrst, pben, gpiob_drvr, gpiob_dircr, gpiob_pur, gpiob_pdr, gpiob_iner, gpiob_odr, [
    PB0: (pb0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PB1: (pb1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PB2: (pb2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PB3: (pb3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PB4: (pb4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PB5: (pb5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PB6: (pb6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PB7: (pb7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    PB8: (pb8, 8, Input<Disabled>, AF0, dir8, pu8, pd8, inen8, od8),
    PB9: (pb9, 9, Input<Disabled>, AF0, dir9, pu9, pd9, inen9, od9),
    PB10: (pb10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PB11: (pb11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    PB12: (pb12, 12, Input<Disabled>, AF0, dir12, pu12, pd12, inen12, od12),
    PB13: (pb13, 13, Input<Disabled>, AF0, dir13, pu13, pd13, inen13, od13),
    PB14: (pb14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PB15: (pb15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOC, gpioc, pcrst, pcen, gpioc_drvr, gpioc_dircr, gpioc_pur, gpioc_pdr, gpioc_iner, gpioc_odr, [
    PC0: (pc0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PC1: (pc1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PC2: (pc2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PC3: (pc3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PC4: (pc4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PC5: (pc5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PC6: (pc6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PC7: (pc7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    PC8: (pc8, 8, Input<Disabled>, AF0, dir8, pu8, pd8, inen8, od8),
    PC9: (pc9, 9, Input<Disabled>, AF0, dir9, pu9, pd9, inen9, od9),
    PC10: (pc10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PC11: (pc11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    PC12: (pc12, 12, Input<Disabled>, AF0, dir12, pu12, pd12, inen12, od12),
    PC13: (pc13, 13, Input<Disabled>, AF0, dir13, pu13, pd13, inen13, od13),
    PC14: (pc14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PC15: (pc15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOD, gpiod, pdrst, pden, gpiod_drvr, gpiod_dircr, gpiod_pur, gpiod_pdr, gpiod_iner, gpiod_odr, [
    PD0: (pd0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PD1: (pd1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PD2: (pd2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
]);
