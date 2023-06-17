use super::GpioRegExt;

gpio_trait!(gpioa);
gpio_trait!(gpiob);
gpio_trait!(gpioc);
gpio_trait!(gpiod);
gpio_trait!(gpioe);

gpio!(GPIOA, gpioa, parst, paen, drvr, dircr, pur, pdr, iner, odr, [
    PA0: (pa0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PA1: (pa1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PA2: (pa2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PA3: (pa3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PA4: (pa4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PA5: (pa5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PA6: (pa6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PA7: (pa7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    PA8: (pa8, 8, Input<Disabled>, AF0, dir8, pu8, pd8, inen8, od8),
    PA9: (pa9, 9, Input<Disabled>, AF0, dir9, pu9, pd9, inen9, od9),
    PA10: (pa10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PA11: (pa11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    PA12: (pa12, 12, Input<Disabled>, AF0, dir12, pu12, pd12, inen12, od12),
    PA13: (pa13, 13, Input<Disabled>, AF0, dir13, pu13, pd13, inen13, od13),
    PA14: (pa14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PA15: (pa15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOB, gpiob, pbrst, pben, drvr, dircr, pur, pdr, iner, odr, [
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

gpio!(GPIOC, gpioc, pcrst, pcen, drvr, dircr, pur, pdr, iner, odr, [
    PC0: (pc0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PC1: (pc1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PC2: (pc2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PC3: (pc3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PC4: (pc4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PC5: (pc5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PC6: (pc6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PC7: (pc7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    // BOOT0
    PC8: (pc8, 8, Input<PullUp>, AF0, dir8, pu8, pd8, inen8, od8),
    // BOOT1
    PC9: (pc9, 9, Input<PullUp>, AF0, dir9, pu9, pd9, inen9, od9),
    PC10: (pc10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PC11: (pc11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    PC12: (pc12, 12, Input<Disabled>, AF0, dir12, pu12, pd12, inen12, od12),
    PC13: (pc13, 13, Input<Disabled>, AF0, dir13, pu13, pd13, inen13, od13),
    PC14: (pc14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PC15: (pc15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOD, gpiod, pdrst, pden, drvr, dircr, pur, pdr, iner, odr, [
    PD0: (pd0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PD1: (pd1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PD2: (pd2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PD3: (pd3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PD4: (pd4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PD5: (pd5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PD6: (pd6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PD7: (pd7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    PD8: (pd8, 8, Input<Disabled>, AF0, dir8, pu8, pd8, inen8, od8),
    PD9: (pd9, 9, Input<Disabled>, AF0, dir9, pu9, pd9, inen9, od9),
    PD10: (pd10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    PD11: (pd11, 11, Input<Disabled>, AF0, dir11, pu11, pd11, inen11, od11),
    PD12: (pd12, 12, Input<Disabled>, AF0, dir12, pu12, pd12, inen12, od12),
    PD13: (pd13, 13, Input<Disabled>, AF0, dir13, pu13, pd13, inen13, od13),
    PD14: (pd14, 14, Input<Disabled>, AF0, dir14, pu14, pd14, inen14, od14),
    PD15: (pd15, 15, Input<Disabled>, AF0, dir15, pu15, pd15, inen15, od15),
]);

gpio!(GPIOE, gpioe, perst, peen, drvr, dircr, pur, pdr, iner, odr, [
    PE0: (pe0, 0, Input<Disabled>, AF0, dir0, pu0, pd0, inen0, od0),
    PE1: (pe1, 1, Input<Disabled>, AF0, dir1, pu1, pd1, inen1, od1),
    PE2: (pe2, 2, Input<Disabled>, AF0, dir2, pu2, pd2, inen2, od2),
    PE3: (pe3, 3, Input<Disabled>, AF0, dir3, pu3, pd3, inen3, od3),
    PE4: (pe4, 4, Input<Disabled>, AF0, dir4, pu4, pd4, inen4, od4),
    PE5: (pe5, 5, Input<Disabled>, AF0, dir5, pu5, pd5, inen5, od5),
    PE6: (pe6, 6, Input<Disabled>, AF0, dir6, pu6, pd6, inen6, od6),
    PE7: (pe7, 7, Input<Disabled>, AF0, dir7, pu7, pd7, inen7, od7),
    PE8: (pe8, 8, Input<Disabled>, AF0, dir8, pu8, pd8, inen8, od8),
    PE9: (pe9, 9, Input<Disabled>, AF0, dir9, pu9, pd9, inen9, od9),
    PE10: (pe10, 10, Input<Disabled>, AF0, dir10, pu10, pd10, inen10, od10),
    // JTDO
    PE11: (pe11, 11, Input<PullDown>, AF0, dir11, pu11, pd11, inen11, od11),
    // JTCK/SWCLK
    PE12: (pe12, 12, Input<PullUp>, AF0, dir12, pu12, pd12, inen12, od12),
    // JTMS/SWDIO
    PE13: (pe13, 13, Input<PullUp>, AF0, dir13, pu13, pd13, inen13, od13),
    // JTDI
    PE14: (pe14, 14, Input<PullUp>, AF0, dir14, pu14, pd14, inen14, od14),
    // JTRS
    PE15: (pe15, 15, Input<PullUp>, AF0, dir15, pu15, pd15, inen15, od15),
]);
