import {Component, OnInit} from '@angular/core';
import {Observable, of, Subscription, throwError} from 'rxjs';
import {environment} from '../../../environments/environment';
import {HttpClient} from '@angular/common/http';
import {catchError, map} from 'rxjs/operators';
import {ActivatedRoute, Router} from '@angular/router';
import {AuthApi} from '../../api/auth.api';
import {CheckAuthenticated, GetAccessTokenResponse, UserInterface} from '../../interface/auth.interface';

@Component({
  selector: 'app-github-callback',
  templateUrl: './github-callback.component.html'
})

export class GithubCallbackComponent implements OnInit {

  constructor(private route: ActivatedRoute,
              private router: Router,
              private httpClient: HttpClient,
              private authApi: AuthApi) {
  }

  public accessToken: any;

  private static handleError(error: Response): Observable<never> {
      return throwError(`GitHub ${error.statusText || 'Server error'}`);
  }

  public getToken(code: string): Observable<GetAccessTokenResponse> {
      return this.authApi.getAccessToken(code);
  }

  ngOnInit() {
      debugger;
      setTimeout(() => {
          debugger;
          this.route.queryParams.subscribe(
              (param: any) => {
                  debugger;
                  const code = param.code;
                  this.authApi.getAccessToken(code).subscribe((result: GetAccessTokenResponse) => {
                      debugger;
                      this.authApi.getUser(result.access_token);
                  }, (err: any) => {
                      debugger;
                      localStorage.removeItem('access_token');
                      GithubCallbackComponent.handleError(err);
                  });
              });
      }, 2000);
  }

}