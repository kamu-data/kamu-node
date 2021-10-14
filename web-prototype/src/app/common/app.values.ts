import {Injectable} from "@angular/core";

@Injectable()
export default class AppValues {
  public static appLogo: string = 'assets/icons/kamu_logo_icon.svg';
  public static urlProfile: string = 'profile';
  public static urlLogin: string = 'login';
  public static urlSearch: string = 'search';
  public static urlDatasetView: string = 'dataset-view';
  public static urlDatasetCreate: string = 'dataset-create';
  public static urlDatasetCreateSelectType: string = 'select-type';
  public static urlDatasetCreateRoot: string = 'root';

  public static capitalizeFirstLetter(text: string): string {
    return text.charAt(0).toUpperCase() + text.slice(1);
  }
  public static isMobileView(): boolean {
    return window.innerWidth < window.innerHeight;
  }
}
