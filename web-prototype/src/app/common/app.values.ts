import {Injectable} from "@angular/core";
import * as moment from 'moment-timezone';

@Injectable()
export default class AppValues {
  public static appLogo: string = 'assets/icons/kamu_logo_icon.svg';
  public static urlProfile: string = 'profile';
  public static urlLogin: string = 'login';
  public static urlSearch: string = 'search';
  public static urlDatasetView: string = 'dataset-view';
  public static urlDatasetViewOverviewType: string = 'overview';
  public static urlDatasetViewMetadataType: string = 'metadata';
  public static urlDatasetCreate: string = 'dataset-create';
  public static urlDatasetCreateSelectType: string = 'select-type';
  public static urlDatasetCreateRoot: string = 'root';


  public static httpPattern: RegExp = new RegExp(/^(http:\/\/)|(https:\/\/)/i);

  public static capitalizeFirstLetter(text: string): string {
    return text.charAt(0).toUpperCase() + text.slice(1);
  }
  public static isMobileView(): boolean {
    return window.innerWidth < window.innerHeight;
  }

  /**
   * Using for ISO format date from server '2021-02-26 00:13:11.959000'
   * This method converts the date from the format ISO used to the format UTC
   * and then to the local format by displaying it in the specified pattern
   * @param dateParams {Date} date new Date('2021-02-26 00:13:11.959000')
   * @param dateParams {string} format 'MMMM DD, YYYY'
   * @param dateParams {boolean} isTextDate for example 'Today', 'Yesterday'
   * @return {string}
   */
   public static momentConverDatetoLocalWithFormat(dateParams: {date: Date, format?: string, isTextDate?: boolean}): string {
       const stringDate: string = new Date(dateParams.date).toString();
       const UTCStringDate: string = stringDate.split('.')[0] + '.000Z';
       const ISOStringDate: string = new Date(UTCStringDate).toISOString();

       if (dateParams.isTextDate) {
           if (moment(dateParams.date).isSame(moment().subtract(1, 'day'), "day")) {
               return 'Yesterday';
           }
           if (moment(dateParams.date).isSame(moment(), "day")) {
               return 'Today';
           }
       }

       return moment(ISOStringDate).format(dateParams.format);
   }

   /**
     * @desc gets current datetime and convert the given date
     * object’s contents into a string in ISO format (ISO 8601)
     * "2014-09-08T08:02:17-05:00"
     * @returns {string}
     */
    public static getDateNowISO8601(): string {
        return moment().format();
    }


    /**
     * Checks if ulr has a protocol (http:// or https://).
     * In case false - adds 'https://' at the beginning of url.
     * @param {string} url
     * @returns {string}
     */
    public static normalizeUrl(url: string): string {
        return AppValues.httpPattern.test(url) ? url : `https://${ url }`;
    }

    public static fixedEncodeURIComponent(text: string): string {
        return encodeURIComponent(text)
            .replace('%2C', ',')
            .replace(/[!'"/*]/g, (c: string) => {
                return '%' + c.charCodeAt(0).toString(16);
            });
    }
    /**
     * Makes deep copy of item without binding to its memory link
     * @param {T} item
     * @returns {T}
     */
    public static deepCopy<T>(item: T): T {
       // @ts-ignore
        let copy;

       if (null == item || "object" !== typeof item) {
          return item;
       }

       if (item instanceof Array) {
           copy = [];
           item.forEach(obj => {
              copy.push(this.deepCopy(obj));
           });
           // @ts-ignore
           return copy;
       }

       if (item instanceof Object) {
           copy = {};
           for (let attr in item) {
               // @ts-ignore
               if (item.hasOwnProperty(attr)) {
                   // @ts-ignore
                  copy[attr] = this.deepCopy(item[attr]);
               }
           }
           // @ts-ignore
           return copy;
       }

       throw new Error("Unable to copy obj! Its type isn't supported.");
    }
}
