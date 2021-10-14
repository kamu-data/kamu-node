import {Component, HostListener, OnInit} from '@angular/core';
import AppValues from "./common/app.values";
import {AppSearchService} from "./search/search.service";
import { filter } from 'rxjs/operators';
import {Router, NavigationEnd} from '@angular/router';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  public appLogo: string = `/${AppValues.appLogo}`;
  public isMobileView: boolean = false;
  public searchValue: string = '';
  public isVisible: boolean = true;
  private appHeaderNotVisiblePages: string[] = [AppValues.urlDatasetCreate, AppValues.urlLogin];

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.checkView();
  }

  constructor(
      private router: Router,
      private appSearchService: AppSearchService,
  ) { }

  public ngOnInit(): void {
    this.checkView();
    this.appHeaderInit();
  }
  private appHeaderInit(): void {
    this.router.events
        .pipe(filter(event => event instanceof NavigationEnd))
        .subscribe((event: any) => {
          this.isVisible = this.isAvailableAppHeaderUrl(event.url);
        });
  }

  private checkView(): void {
    this.isMobileView = AppValues.isMobileView();
  }
  private isAvailableAppHeaderUrl(url: string): boolean {
     return !this.appHeaderNotVisiblePages.some(item => url.toLowerCase().includes(item));
  }
  public onInputSearch(searchValue: string) {
    this.router.navigate(['search']);
    this.appSearchService.searchChanges(searchValue);
    this.appSearchService.search(searchValue);
  }

  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }

  public onAddNew(): void {
    this.router.navigate(['dataset-create'])
  }
}
