import {Injectable} from "@angular/core";
import * as moment from 'moment-timezone';

@Injectable()
export default class AppValues {
  public static appLogo = 'assets/icons/kamu_logo_icon.svg';
  public static urlProfile = 'profile';
  public static urlLogin = 'login';
  public static urlGithubCallback = 'github_callback';
  public static urlSearch = 'search';
  public static urlDatasetView = 'dataset-view';
  public static urlDatasetViewOverviewType = 'overview';
  public static urlDatasetViewMetadataType = 'metadata';
  public static urlDatasetCreate = 'dataset-create';
  public static urlDatasetCreateSelectType = 'select-type';
  public static urlDatasetCreateRoot = 'root';

  public static localStorageCode = 'code';
  public static localStorageAccessToken = 'code';


  public static httpPattern = new RegExp(/^(http:\/\/)|(https:\/\/)/i);

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
    /* eslint-disable  @typescript-eslint/no-explicit-any, @typescript-eslint/ban-ts-comment */
    // @ts-ignore
    public static deepCopy<T>(item: T): any {
        /* eslint-disable  @typescript-eslint/no-explicit-any */
        let copy: any;

       if (null == item || "object" !== typeof item) {
          /* eslint-disable  @typescript-eslint/no-explicit-any */
          return item;
       }

       if (item instanceof Array) {
           copy = [];
           /* eslint-disable  @typescript-eslint/no-explicit-any */
           item.forEach((obj: any) => {
               return (copy as Array<any>).push(this.deepCopy(obj));
           });

           return copy;
       }

       if (item instanceof Object) {
           copy = {};
           for (const attr in item) {
               /* eslint-disable  @typescript-eslint/no-explicit-any, @typescript-eslint/ban-ts-comment, no-prototype-builtins */
               // @ts-ignore
               if ((item as any).hasOwnProperty(attr)) {
                   /* eslint-disable  @typescript-eslint/no-explicit-any */
                   (copy as any)[attr] = this.deepCopy(item[attr]);
               }
           }

           return copy;
       }

       throw new Error("Unable to copy obj! Its type isn't supported.");
    }
    /* eslint-disable  @typescript-eslint/no-explicit-any, @typescript-eslint/ban-ts-comment */
    // @ts-ignore
    public static shellSort(arr: any[]) {
     const n: number = arr.length;

     //Start with a really large gap, and then reduce the gap until there isn't any
     //With this, the gap starts as half of the array length, and then half of that every time
     for (let gap = Math.floor(n / 2); gap > 0; gap = Math.floor(gap / 2)) {
         //Do a insertion sort for each of the section the gap ends up dividing
         for (let i = gap; i < n; i += 1) {
             //We store the current varible
             const temp = arr[i];

             //This is the insection sort to sort the section into order
             let j;
             for (j = i; j >= gap && arr[j - gap] > temp; j -= gap) {
                 arr[j] = arr[j - gap];
             }

             arr[j] = temp;
         }
     }

     return arr;
 }
}
