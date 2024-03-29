// Font class declaration
export class Font {
    /** create a new font instance **/
    constructor(scale: number, buffer: Uint8Array);

    /** free font instance **/
    free(): void;
}

// Captcha class declaration
export class Captcha {
    /** create a new captcha instance **/
    constructor(font: Font, width: number, height: number, chars?: number);

    /** free captcha instance **/
    free(): void;

    /** return the raw unencoded bytes of the captcha image **/
    get buffer(): Uint8Array;

    /** returns the solution of the captcha **/
    get solution(): string;

    /** returns the captcha image already encoded as a png */
    png(): Uint8Array;
}
