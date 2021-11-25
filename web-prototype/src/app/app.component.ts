import {Component, HostListener, OnInit} from '@angular/core';
import AppValues from './common/app.values';
import {AppSearchService} from './search/search.service';
import {filter} from 'rxjs/operators';
import {Router, NavigationEnd, ActivatedRoute} from '@angular/router';
import {DatasetIDsInterface, TypeNames} from './interface/search.interface';
import {AuthApi} from './api/auth.api';
import {UserInterface} from './interface/auth.interface';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  public appLogo = `/${AppValues.appLogo}`;
  public isMobileView = false;
  public searchValue: any = '';
  public isVisible = true;
  public user: UserInterface;
  private appHeaderNotVisiblePages: string[] = [AppValues.urlDatasetCreate, AppValues.urlLogin, AppValues.urlGithubCallback];
  private _window: Window;

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.checkView();
  }

  constructor(
      private route: ActivatedRoute,
      private router: Router,
      private appSearchService: AppSearchService,
      private authApi: AuthApi
  ) {
    this._window = window;
  }

  public ngOnInit(): void {
    this.checkView();
    this.appHeaderInit();
    this.authApi.onUserChanges.subscribe((user: UserInterface | {}) => {
      this.user = AppValues.deepCopy(user);
    });
    this.authentification();
  }

  authentification(): void {
    const code: string | null = localStorage.getItem(AppValues.localStorageCode);

    if (location.href.includes(AppValues.urlLogin) || location.href.includes(AppValues.urlGithubCallback)) {
      return;
    } else {
      if (typeof code === 'string' && !this.authApi.isAuthUser) {
        this.authApi.getUserInfoAndToken(code).subscribe();
        return;
      }
    }
  }

  private appHeaderInit(): void {
    this.appSearchService.onSearchChanges.subscribe((searchValue: string) => {
        this.searchValue = searchValue;
    });

    /* eslint-disable  @typescript-eslint/no-explicit-any */
    this.router.events
        .pipe(filter(event => event instanceof NavigationEnd))
        .subscribe((event: any) => {
          this.isVisible = this.isAvailableAppHeaderUrl(event.url);

          if (event.url.split('?id=').length > 1) {
              const searchValue: string = AppValues.fixedEncodeURIComponent(event.url.split('?id=')[1].split('&')[0]);
              this.appSearchService.searchChanges(searchValue);
          }
        });
  }

  private checkView(): void {
    this.isMobileView = AppValues.isMobileView();
  }
  private isAvailableAppHeaderUrl(url: string): boolean {
     return !this.appHeaderNotVisiblePages.some(item => url.toLowerCase().includes(item));
  }

  public onSelectDataset(item: DatasetIDsInterface): void {
    if (item.__typename === TypeNames.datasetType) {
      this.router.navigate([AppValues.urlDatasetView], {queryParams: {id: item.id, type: AppValues.urlDatasetViewOverviewType}});
    } else {
      this.router.navigate([AppValues.urlSearch], {queryParams: {id: item.id, p: 1}});
    }
  }

  public onOpenUserInfo(): void {
    // tslint:disable-next-line:no-console
    console.info('click onOpenUserInfo');
  }

  public onAddNew(): void {
    this.router.navigate(['dataset-create']);
  }

  public onLogin(): void {
    this.router.navigate([AppValues.urlLogin]);
  }
  public onLogOut(): void {
    this.authApi.logOut();
  }
}
